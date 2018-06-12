use combine::{ParseResult, parser, Parser};
use combine::{choice};

use ast::{Item};
use helpers::{semi, ident};
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
    )).parse_stream(input)
}
