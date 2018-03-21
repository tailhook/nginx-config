use combine::{eof, many1, ParseResult, parser, Parser};
use combine::{choice, position};

use ast::{Main, Directive, Item};
use helpers::{semi, ident, string};

use tokenizer::TokenStream;
use error::ParseError;

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

pub fn directive<'a>(input: &mut TokenStream<'a>)
    -> ParseResult<Directive, TokenStream<'a>>
{
    position()
    .and(choice((
        ident("daemon").with(parser(bool)).skip(semi())
            .map(Item::Daemon),
        ident("master_process").with(parser(bool)).skip(semi())
            .map(Item::MasterProcess),
        parser(worker_processes)
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
