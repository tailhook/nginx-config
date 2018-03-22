use std::path::PathBuf;

use combine::{eof, many, many1, ParseResult, parser, Parser};
use combine::{choice, position};
use combine::error::StreamError;
use combine::easy::Error;

use ast::{self, Main, Directive, Item};
use error::ParseError;
use helpers::{semi, ident, string, prefix};
use position::Pos;
use tokenizer::TokenStream;
use value::Value;


pub fn value<'a>(input: &mut TokenStream<'a>)
    -> ParseResult<Value, TokenStream<'a>>
{
    (position(), string())
    .and_then(|(p, v)| Value::parse(p, v))
    .parse_stream(input)
}


pub fn directives<'a>(input: &mut TokenStream<'a>)
    -> ParseResult<Item, TokenStream<'a>>
{
    choice((
        ident("proxy_pass").with(parser(value)).skip(semi())
            .map(Item::ProxyPass),
        ident("proxy_set_header").with(parser(value)).and(parser(value))
            .skip(semi())
            .map(|(field, value)| Item::ProxySetHeader { field, value }),
    )).parse_stream(input)
}
