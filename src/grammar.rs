use std::path::PathBuf;

use combine::{eof, many, many1, ParseResult, parser, Parser};
use combine::{choice, position, optional};
use combine::error::StreamError;
use combine::easy::Error;

use ast::{self, Main, Directive, Item};
use error::ParseError;
use gzip;
use helpers::{semi, ident, text, string, prefix};
use position::Pos;
use proxy;
use tokenizer::{TokenStream, Token};
use value::Value;


pub fn bool<'a>(input: &mut TokenStream<'a>)
    -> ParseResult<bool, TokenStream<'a>>
{
    choice((
        ident("on").map(|_| true),
        ident("off").map(|_| false),
    ))
    .parse_stream(input)
}

pub fn value<'a>(input: &mut TokenStream<'a>)
    -> ParseResult<Value, TokenStream<'a>>
{
    (position(), string())
    .and_then(|(p, v)| Value::parse(p, v))
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

enum ListenParts {
    DefaultServer,
    Ssl,
    Ext(ast::HttpExt),
    ProxyProtocol,
    SetFib(i32),
    FastOpen(u32),
    Backlog(i32),
    RcvBuf(u64),
    SndBuf(u64),
    Deferred,
    Bind,
    Ipv6Only(bool),
    ReusePort,
}

pub fn listen<'a>(input: &mut TokenStream<'a>)
    -> ParseResult<Item, TokenStream<'a>>
{
    use ast::{Address, Listen, HttpExt};
    use self::ListenParts::*;

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
    .and(many::<Vec<_>, _>(choice((
        ident("default_server").map(|_| DefaultServer),
        ident("ssl").map(|_| Ssl),
        ident("http2").map(|_| Ext(HttpExt::Http2)),
        ident("spdy").map(|_| Ext(HttpExt::Spdy)),
        ident("proxy_protocol").map(|_| ProxyProtocol),
        prefix("setfib=").and_then(|val| val.parse().map(SetFib)),
        prefix("fastopen=").and_then(|val| val.parse().map(FastOpen)),
        prefix("backlog=").and_then(|val| val.parse().map(Backlog)),
        prefix("rcvbuf=").and_then(|val| val.parse().map(RcvBuf)),
        prefix("sndbuf=").and_then(|val| val.parse().map(SndBuf)),
        ident("deferred").map(|_| Deferred),
        ident("bind").map(|_| Bind),
        prefix("ipv6only=").and_then(|val| Ok(Ipv6Only(match val {
            "on" => true,
            "off" => false,
            _ => return Err(Error::unexpected_message("only on/off supported")),
        }))),
        ident("reuseport").map(|_| ReusePort),
    ))))
    .map(|(addr, items)| {
        let mut lst = Listen::new(addr);
        for item in items {
            match item {
                DefaultServer => lst.default_server = true,
                Ssl => lst.ssl = true,
                Ext(ext) => lst.ext = Some(ext),
                ProxyProtocol => lst.proxy_protocol = true,
                SetFib(v) => lst.setfib = Some(v),
                FastOpen(v) => lst.fastopen = Some(v),
                Backlog(v) => lst.backlog = Some(v),
                RcvBuf(v) => lst.rcvbuf = Some(v),
                SndBuf(v) => lst.sndbuf = Some(v),
                Deferred => lst.deferred = true,
                Bind => lst.bind = true,
                Ipv6Only(v) => lst.ipv6only = Some(v),
                ReusePort => lst.reuseport = true,
            }
        }
        return lst;
    })
    .skip(semi())
    .map(Item::Listen)
    .parse_stream(input)
}

pub fn add_header<'a>(input: &mut TokenStream<'a>)
    -> ParseResult<Item, TokenStream<'a>>
{
    ident("add_header")
    .with((
        parser(value),
        parser(value),
        optional(ident("always").map(|_| ())),
    )).map(|(field, value, always)| {
        ast::AddHeader { field, value, always: always.is_some() }
    })
    .skip(semi())
    .map(Item::AddHeader)
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

// A string that forbids variables
pub fn raw<'a>(input: &mut TokenStream<'a>)
    -> ParseResult<String, TokenStream<'a>>
{
    // TODO(tailhook) unquote single and double quotes
    // error on variables?
    string().and_then(|t| Ok::<_, Error<_, _>>(t.value.to_string()))
    .parse_stream(input)
}

pub fn location<'a>(input: &mut TokenStream<'a>)
    -> ParseResult<Item, TokenStream<'a>>
{
    use ast::LocationPattern::*;
    ident("location").with(choice((
        text("=").with(parser(raw).map(Exact)),
        text("^~").with(parser(raw).map(FinalPrefix)),
        text("~").with(parser(raw).map(Regex)),
        text("~*").with(parser(raw).map(RegexInsensitive)),
        parser(raw)
            .map(|v| if v.starts_with('*') {
                Named(v)
            } else {
                Prefix(v)
            }),
    ))).and(parser(block))
    .map(|(pattern, (position, directives))| {
        Item::Location(ast::Location { pattern, position, directives })
    })
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
        ident("root").with(parser(value)).skip(semi()).map(Item::Root),
        ident("alias").with(parser(value)).skip(semi()).map(Item::Alias),
        parser(location),
        parser(listen),
        parser(add_header),
        parser(proxy::directives),
        parser(gzip::directives),
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
