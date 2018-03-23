use combine::{many1, ParseResult, parser, Parser};
use combine::{choice};

use ast::{Item};
use grammar::bool;
use helpers::{semi, ident};
use tokenizer::TokenStream;

pub fn gzip_static<'a>(input: &mut TokenStream<'a>)
    -> ParseResult<Item, TokenStream<'a>>
{
    use ast::GzipStatic::*;
    ident("gzip_static").with(choice((
        ident("on").map(|_| On),
        ident("off").map(|_| Off),
        ident("always").map(|_| Always),
    )))
    .map(Item::GzipStatic)
    .skip(semi())
    .parse_stream(input)
}

pub fn gzip_proxied<'a>(input: &mut TokenStream<'a>)
    -> ParseResult<Item, TokenStream<'a>>
{
    use ast::GzipProxied::*;
    ident("gzip_proxied").with(many1(choice((
        ident("off").map(|_| Off),
        ident("expired").map(|_| Expired),
        ident("no-cache").map(|_| NoCache),
        ident("no-store").map(|_| NoStore),
        ident("private").map(|_| Private),
        ident("no_last_modified").map(|_| NoLastModified),
        ident("no_etag").map(|_| NoEtag),
        ident("auth").map(|_| Auth),
        ident("any").map(|_| Any),
    ))))
    .map(Item::GzipProxied)
    .skip(semi())
    .parse_stream(input)
}

pub fn directives<'a>(input: &mut TokenStream<'a>)
    -> ParseResult<Item, TokenStream<'a>>
{
    choice((
        ident("gzip").with(parser(bool)).skip(semi())
            .map(Item::Gzip),
        parser(gzip_static),
        parser(gzip_proxied),
    )).parse_stream(input)
}
