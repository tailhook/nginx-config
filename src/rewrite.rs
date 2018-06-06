use combine::{parser, Parser};
use combine::{choice, optional};

use ast::{self, Item};
use grammar::value;
use helpers::{semi, ident, string};
use tokenizer::{TokenStream};


fn rewrite<'a>()
    -> impl Parser<Output=Item, Input=TokenStream<'a>>
{
    use ast::RewriteFlag::*;
    use ast::Item::Rewrite;

    ident("rewrite")
    .with(string())
    .and(parser(value))
    .and(optional(choice((
        ident("last").map(|_| Last),
        ident("break").map(|_| Break),
        ident("redirect").map(|_| Redirect),
        ident("permanent").map(|_| Permanent),
    ))))
    .map(|((regex, replacement), flag)| {
        Rewrite(ast::Rewrite {
            regex: regex.value.to_string(), replacement, flag,
        })
    })
    .skip(semi())
}

pub fn directives<'a>()
    -> impl Parser<Output=Item, Input=TokenStream<'a>>
{
    choice((
        rewrite(),
    ))
}
