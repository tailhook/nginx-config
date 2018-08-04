use combine::easy::{Errors, Error};

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

#[cfg(not(feature="fuzzy_errors"))]
impl<'a> From<InternalError<'a>> for ParseError {
    fn from(e: InternalError<'a>) -> ParseError {
        ParseError(e
            .map_token(|t| t.value.to_string())
            .map_range(|t| t.value.to_string()))
    }
}

fn convert(error: Error<Token, Token>) -> Error<String, String> {
    error
    .map_token(|t| t.value.to_string())
    .map_range(|t| t.value.to_string())
}

#[cfg(feature="fuzzy_errors")]
impl<'a> From<InternalError<'a>> for ParseError {
    fn from(e: InternalError<'a>) -> ParseError {
        use strsim::jaro_winkler;
        use combine::easy::{Info};

        let mut error_buf = Vec::new();
        let mut expected_buf = Vec::new();
        let mut unexpected = None;
        // Note: we assume that "expected" will go after error
        //       in output and that's fine
        for item in e.errors {
            match item {
                Error::Expected(info) => {
                    expected_buf.push(info);
                    continue;
                }
                Error::Unexpected(ref val) => {
                    unexpected = Some(val.to_string());
                }
                _ => {}
            }
            error_buf.push(convert(item));
        }
        println!("Unexpected {:?}, expected {:?}", unexpected, expected_buf);
        if let Some(unexpected) = unexpected {
            if expected_buf.len() > 3 {
                let mut close = Vec::new();
                for item in &expected_buf {
                    match item {
                        Info::Borrowed(item) => {
                            let conf = jaro_winkler(&unexpected, item);
                            if conf > 0.8 {
                                close.push((item, conf));
                            }
                        }
                        _ => {
                            // assuming any other thing is just a text, not
                            // expected token
                        }
                    }
                }
                close.sort_by_key(|&(_, ref x)| (10000. - 10000. * x) as u32);
                close.truncate(3);
                for (item, _) in &close {
                    error_buf.push(convert(Error::Expected(
                        Info::Borrowed(item))));
                }
                if close.len() < expected_buf.len() {
                    error_buf.push(Error::Expected(Info::Owned(format!(
                        "one of {} options",
                        expected_buf.len() - close.len(),
                    ))));
                }
            } else {
                for e in expected_buf {
                    error_buf.push(convert(Error::Expected(e)));
                }
            }
        } else {
            for e in expected_buf {
                error_buf.push(convert(Error::Expected(e)));
            }
        }
        return ParseError(Errors { position: e.position, errors: error_buf })
    }
}
