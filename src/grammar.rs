use std::path::PathBuf;
use std::net::SocketAddr;

use combine::{eof, many, many1, ParseResult, parser, Parser};
use combine::{choice, position};

use ast::{self, Main, Directive, Item};
use error::ParseError;
use helpers::{semi, ident, string};
use position::Pos;
use tokenizer::TokenStream;


pub fn bool<'a>(input: &mut TokenStream<'a>)
    -> ParseResult<bool, TokenStream<'a>>
{
    choice((
        ident("on").map(|_| true),
        ident("off").map(|_| false),
    ))
    .parse_stream(input)
}

pub fn worker_processes<'a>(input: &mut TokenStream<'a>)
    -> ParseResult<Item, TokenStream<'a>>
{
    use ast::WorkerProcesses;
    ident("worker_processes")
    .with(choice((
        ident("auto").map(|_| WorkerProcesses::Auto),
        string().and_then(|s| s.value.parse().map(WorkerProcesses::Exact)),
    )))
    .skip(semi())
    .map(Item::WorkerProcesses)
    .parse_stream(input)
}

pub fn listen<'a>(input: &mut TokenStream<'a>)
    -> ParseResult<Item, TokenStream<'a>>
{
    use ast::Listen;
    use ast::Address;

    ident("listen")
    .with(string().and_then(|s| -> Result<_, ::combine::easy::Error<_, _>> {
        let v = if s.value.starts_with("unix:") {
            Address::Unix(PathBuf::from(&s.value[6..]))
        } else if s.value.starts_with("*:") {
            Address::StarPort(s.value[2..].parse()?)
        } else {
            s.value.parse().map(Address::Port)
            .or_else(|_| s.value.parse().map(Address::Ip))?
        };
        Ok(v)
    }))
    .map(|a| Listen::new(a))
    .skip(semi())
    .map(Item::Listen)
    .parse_stream(input)
}

pub fn block<'a>(input: &mut TokenStream<'a>)
    -> ParseResult<((Pos, Pos), Vec<Directive>), TokenStream<'a>>
{
    use tokenizer::Kind::{BlockStart, BlockEnd};
    use helpers::kind;
    (
        position(),
        kind(BlockStart)
            .with(many(parser(directive)))
            .skip(kind(BlockEnd)),
        position(),
    )
    .map(|(s, dirs, e)| ((s, e), dirs))
    .parse_stream(input)
}

pub fn directive<'a>(input: &mut TokenStream<'a>)
    -> ParseResult<Directive, TokenStream<'a>>
{
    position()
    .and(choice((
        ident("daemon").with(parser(bool)).skip(semi())
            .map(Item::Daemon),
        ident("master_process").with(parser(bool)).skip(semi())
            .map(Item::MasterProcess),
        parser(worker_processes),
        ident("http").with(parser(block))
            .map(|(position, directives)| ast::Http { position, directives })
            .map(Item::Http),
        ident("server").with(parser(block))
            .map(|(position, directives)| ast::Server { position, directives })
            .map(Item::Server),
        parser(listen),
    )))
    .map(|(pos, dir)| Directive {
        position: pos,
        item: dir,
    })
    .parse_stream(input)
}


/// Parses a piece of config in "main" context (i.e. top-level)
pub fn parse_main(s: &str) -> Result<Main, ParseError> {
    let mut tokens = TokenStream::new(s);
    let (doc, _) = many1(parser(directive))
        .map(|d| Main { directives: d })
        .skip(eof())
        .parse_stream(&mut tokens)
        .map_err(|e| e.into_inner().error)?;

    Ok(doc)
}
