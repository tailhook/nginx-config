use combine::{eof, many1, ParseResult, parser, Parser};
use combine::{choice, position};

use ast::{Main, Directive, Item};
use helpers::{semi, ident};

use tokenizer::TokenStream;
use error::ParseError;


pub fn directive<'a>(input: &mut TokenStream<'a>)
    -> ParseResult<Directive, TokenStream<'a>>
{
    position()
    .and(choice((
        ident("daemon")
            .with(choice((
                ident("on").map(|_| true),
                ident("off").map(|_| false),
            )))
            .skip(semi())
            .map(Item::Daemon),

    )))
    .map(|(pos, dir)| Directive {
        position: pos,
        item: dir,
    })
    .parse_stream(input)
}


/// Parses a piece of config in "main" context (i.e. top-level)
pub fn parse_main(s: &str) -> Result<Main, ParseError> {
    let mut tokens = TokenStream::new(s);
    let (doc, _) = many1(parser(directive))
        .map(|d| Main { directives: d })
        .skip(eof())
        .parse_stream(&mut tokens)
        .map_err(|e| e.into_inner().error)?;

    Ok(doc)
}
