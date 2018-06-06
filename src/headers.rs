use combine::{parser, Parser, optional};
use combine::{choice};

use ast::{self, Item};
use grammar::{value};
use helpers::{semi, ident};
use tokenizer::{TokenStream};


fn add_header<'a>()
    -> impl Parser<Output=Item, Input=TokenStream<'a>>
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
}

pub fn directives<'a>()
    -> impl Parser<Output=Item, Input=TokenStream<'a>>
{
    choice((
        add_header(),
    ))
}
