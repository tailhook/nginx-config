use combine::easy::Errors;

use tokenizer::Token;
use position::Pos;

pub type InternalError<'a> = Errors<Token<'a>, Token<'a>, Pos>;


/// Error parsing config
///
/// This structure is opaque for forward compatibility. We are exploring a
/// way to improve both error message and API.
#[derive(Fail, Debug)]
#[fail(display="parse error: {}", _0)]
pub struct ParseError(Errors<String, String, Pos>);

impl<'a> From<InternalError<'a>> for ParseError {
    fn from(e: InternalError<'a>) -> ParseError {
        ParseError(e
            .map_token(|t| t.value.to_string())
            .map_range(|t| t.value.to_string()))
    }
}
