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


pub fn directives<'a>(input: &mut TokenStream<'a>)
    -> ParseResult<Item, TokenStream<'a>>
{
    ident("proxy_pass")
    .with((position(), string()).and_then(|(p, v)| Value::parse(p, v)))
    .skip(semi())
    .map(Item::ProxyPass)
    .parse_stream(input)
}
