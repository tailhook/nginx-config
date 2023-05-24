use combine::{Parser, optional};
use combine::{choice};

use crate::ast::{self, Item};
use crate::grammar::{value};
use crate::helpers::{semi, ident};
use crate::tokenizer::{TokenStream};

fn add_header<'a>()
    -> impl Parser<Output=Item, Input=TokenStream<'a>>
{
    ident("add_header")
    .with((
        value(),
        value(),
        optional(ident("always").map(|_| ())),
    )).map(|(field, value, always)| {
        ast::AddHeader { field, value, always: always.is_some() }
    })
    .skip(semi())
    .map(Item::AddHeader)
}

fn expires<'a>()
    -> impl Parser<Output=Item, Input=TokenStream<'a>>
{
    ident("expires")
    .with(optional(ident("modified"))).map(|x| x.is_some())
    .and(value())
    .map(|(modified, value)| {
        Item::Expires(ast::Expires { modified, value })
    }).skip(semi())
}

pub fn directives<'a>()
    -> impl Parser<Output=Item, Input=TokenStream<'a>>
{
    choice((
        add_header(),
        expires(),
    ))
}
