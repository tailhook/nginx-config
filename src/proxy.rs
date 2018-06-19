use combine::{ParseResult, parser, Parser};
use combine::{choice, many1};
use combine::error::StreamError;
use combine::easy::Error;

use ast::{self, Item};
use helpers::{semi, ident, string};
use tokenizer::TokenStream;
use grammar::{value, bool};


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
        ident("proxy_hide_header").with(parser(value)).skip(semi())
            .map(Item::ProxyHideHeader),
        ident("proxy_pass_header").with(parser(value)).skip(semi())
            .map(Item::ProxyPassHeader),
        ident("proxy_pass_request_headers").with(parser(bool)).skip(semi())
            .map(Item::ProxyPassRequestHeaders),
        ident("proxy_pass_request_body").with(parser(bool)).skip(semi())
            .map(Item::ProxyPassRequestBody),
        ident("proxy_ignore_headers").with(many1(string())).skip(semi())
            .map(|v: Vec<_>| {
                v.into_iter().map(|v| v.value.to_string()).collect()
            })
            .map(Item::ProxyIgnoreHeaders),
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
