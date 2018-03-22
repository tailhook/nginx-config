use std::path::PathBuf;

use combine::{eof, many, many1, ParseResult, parser, Parser};
use combine::{choice, position};
use combine::error::StreamError;
use combine::easy::Error;

use ast::{self, Main, Directive, Item};
use error::ParseError;
use grammar::bool;
use helpers::{semi, ident, string, prefix};
use position::Pos;
use tokenizer::TokenStream;
use value::Value;


pub fn directives<'a>(input: &mut TokenStream<'a>)
    -> ParseResult<Item, TokenStream<'a>>
{
    use ast::GzipStatic::*;
    choice((
        ident("gzip").with(parser(bool)).skip(semi())
            .map(Item::Gzip),
        ident("gzip_static").with(choice((
            ident("on").map(|_| On),
            ident("off").map(|_| Off),
            ident("always").map(|_| Always),
        )))
        .map(Item::GzipStatic)
        .skip(semi())
    )).parse_stream(input)
}
