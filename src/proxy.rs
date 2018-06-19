use combine::{ParseResult, parser, Parser};
use combine::{choice};
use combine::error::StreamError;
use combine::easy::Error;

use ast::{self, Item};
use helpers::{semi, ident, string};
use tokenizer::TokenStream;
use grammar::value;


pub fn directives<'a>(input: &mut TokenStream<'a>)
    -> ParseResult<Item, TokenStream<'a>>
{
    choice((
        ident("proxy_pass").with(parser(value)).skip(semi())
            .map(Item::ProxyPass),
        ident("proxy_set_header").with(parser(value)).and(parser(value))
            .skip(semi())
            .map(|(field, value)| Item::ProxySetHeader { field, value }),
        ident("proxy_method").with(parser(value)).skip(semi())
            .map(Item::ProxyMethod),
        ident("proxy_http_version")
            .with(string()).and_then(|v| {
                match v.value {
                    "1.0" => Ok(ast::ProxyHttpVersion::V1_0),
                    "1.1" => Ok(ast::ProxyHttpVersion::V1_1),
                    _ => Err(Error::unexpected_message("invalid variable")),
                }
            })
            .skip(semi())
            .map(Item::ProxyHttpVersion),
    )).parse_stream(input)
}
