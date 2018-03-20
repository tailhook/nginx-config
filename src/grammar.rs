use combine::{eof, many1, ParseResult, parser, Parser};

use ast::{Main, Directive};

use tokenizer::TokenStream;
use error::ParseError;


pub fn directive<'a>(input: &mut TokenStream<'a>)
    -> ParseResult<Directive, TokenStream<'a>>
{
    /*
    choice((
        parser(schema).map(Definition::SchemaDefinition),
    )).parse_stream(input)
    */
    unimplemented!();
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
