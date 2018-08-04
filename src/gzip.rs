use combine::{many1, Parser};
use combine::{choice};

use ast::{Item};
use grammar::bool;
use helpers::{semi, ident};
use tokenizer::TokenStream;

pub fn gzip_static<'a>() -> impl Parser<Output=Item, Input=TokenStream<'a>> {
    use ast::GzipStatic::*;
    ident("gzip_static").with(choice((
        ident("on").map(|_| On),
        ident("off").map(|_| Off),
        ident("always").map(|_| Always),
    )))
    .map(Item::GzipStatic)
    .skip(semi())
}

pub fn gzip_proxied<'a>() -> impl Parser<Output=Item, Input=TokenStream<'a>> {
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
}

pub fn directives<'a>() -> impl Parser<Output=Item, Input=TokenStream<'a>> {
    choice((
        ident("gzip").with(bool()).skip(semi())
            .map(Item::Gzip),
        gzip_static(),
        gzip_proxied(),
    ))
}
