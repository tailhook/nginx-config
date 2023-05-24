use std::fmt;

use combine::{StreamOnce, Positioned};
use combine::error::{StreamError};
use combine::stream::{Resetable};
use combine::easy::{Error, Errors};

use crate::position::Pos;


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Kind {
    String,
    Semicolon,
    BlockStart,
    BlockEnd,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Token<'a> {
    pub kind: Kind,
    pub value: &'a str,
}

#[derive(Debug, PartialEq)]
pub struct TokenStream<'a> {
    buf: &'a str,
    position: Pos,
    off: usize,
    next_state: Option<(usize, Token<'a>, usize, Pos)>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Checkpoint {
    position: Pos,
    off: usize,
}

impl<'a> StreamOnce for TokenStream<'a> {
    type Item = Token<'a>;
    type Range = Token<'a>;
    type Position = Pos;
    type Error = Errors<Token<'a>, Token<'a>, Pos>;

    fn uncons(&mut self) -> Result<Self::Item, Error<Token<'a>, Token<'a>>> {
        if let Some((at, tok, off, pos)) = self.next_state {
            if at == self.off {
                self.off = off;
                self.position = pos;
                return Ok(tok);
            }
        }
        let old_pos = self.off;
        let (kind, len) = self.peek_token()?;
        let value = &self.buf[self.off-len..self.off];
        self.skip_whitespace();
        let token = Token { kind, value };
        self.next_state = Some((old_pos, token, self.off, self.position));
        Ok(token)
    }
}

impl<'a> Positioned for TokenStream<'a> {
    fn position(&self) -> Self::Position {
        self.position
    }
}

impl<'a> Resetable for TokenStream<'a> {
    type Checkpoint = Checkpoint;
    fn checkpoint(&self) -> Self::Checkpoint {
        Checkpoint {
            position: self.position,
            off: self.off,
        }
    }
    fn reset(&mut self, checkpoint: Checkpoint) {
        self.position = checkpoint.position;
        self.off = checkpoint.off;
    }
}

impl<'a> TokenStream<'a> {
    pub fn new(s: &str) -> TokenStream {
        let mut me = TokenStream {
            buf: s,
            position: Pos { line: 1, column: 1 },
            off: 0,
            next_state: None,
        };
        me.skip_whitespace();
        me
    }

    fn peek_token(&mut self)
        -> Result<(Kind, usize), Error<Token<'a>, Token<'a>>>
    {
        use self::Kind::*;
        let mut iter = self.buf[self.off..].char_indices();
        let cur_char = match iter.next() {
            Some((_, x)) => x,
            None => return Err(Error::end_of_input()),
        };

        match cur_char {
            '{' => {
                self.position.column += 1;
                self.off += 1;
                Ok((BlockStart, 1))
            }
            '}' => {
                self.position.column += 1;
                self.off += 1;
                Ok((BlockEnd, 1))
            }
            ';' => {
                self.position.column += 1;
                self.off += 1;
                Ok((Semicolon, 1))
            }
            '"' | '\'' => {
                let open_quote = cur_char;
                let mut prev_char = cur_char;
                let mut nchars = 1;
                for (idx, cur_char) in iter {
                    nchars += 1;
                    match cur_char {
                        x if x == open_quote && prev_char != '\\' => {
                            self.position.column += nchars;
                            self.off += idx+1;
                            return Ok((String, idx+1));
                        }
                        '\n' => {
                            return Err(
                                Error::unexpected_message(
                                    "unterminated string value"
                                )
                            );
                        }
                        _ => {

                        }
                    }
                    prev_char = cur_char;
                }
                Err(Error::unexpected_message("unterminated string value"))
            }
            _ => {  // any other non-whitespace char is also a token
                let mut prev_char = cur_char;
                let mut nchars = 1;
                while let Some((idx, cur_char)) = iter.next() {
                    match cur_char {
                        '\r' | '\t' | '\n' => {
                            self.position.column += nchars;
                            self.off += idx;
                            return Ok((String, nchars));
                        }
                        '{' if prev_char == '$' => {
                            while let Some((_, cur_char)) = iter.next() {
                                nchars += 1;
                                if cur_char == '}' {
                                    break;
                                }
                            }
                            // TODO(tailhook) validate end of file
                        }
                        ';' | '{' | '}' | ' ' |
                        '\"' | '\'' => {
                            if prev_char == '\\' {
                            } else {
                                self.position.column += nchars;
                                self.off += idx;
                                return Ok((String, nchars));
                            }
                        }
                        '\\' if prev_char == '\\' => {
                            prev_char = ' ';  // reset pending escape
                            nchars += 1;
                            continue;
                        }
                        _ => {}
                    }
                    nchars += 1;
                    prev_char = cur_char;
                }
                let len = self.buf.len() - self.off;
                self.position.column += nchars;
                self.off += len;
                return Ok((String, nchars));
            }
        }
    }

    fn skip_whitespace(&mut self) {
        let mut iter = self.buf[self.off..].char_indices();
        let idx = loop {
            let (idx, cur_char) = match iter.next() {
                Some(pair) => pair,
                None => break self.buf.len() - self.off,
            };
            match cur_char {
                '\u{feff}' | '\r' => continue,
                '\t' => self.position.column += 8,
                '\n' => {
                    self.position.column = 1;
                    self.position.line += 1;
                }
                ' ' => {
                    self.position.column += 1;
                    continue;
                }
                //comment
                '#' => {
                    while let Some((_, cur_char)) = iter.next() {
                        if cur_char == '\r' || cur_char == '\n' {
                            self.position.column = 1;
                            self.position.line += 1;
                            break;
                        }
                    }
                    continue;
                }
                _ => break idx,
            }
        };
        self.off += idx;
    }
}

impl<'a> fmt::Display for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[cfg(test)]
mod test {
    use super::{Kind, TokenStream};
    use super::Kind::*;
    use combine::easy::Error;

    use combine::{StreamOnce, Positioned};

    fn tok_str(s: &str) -> Vec<&str> {
        let mut r = Vec::new();
        let mut s = TokenStream::new(s);
        loop {
            match s.uncons() {
                Ok(x) => r.push(x.value),
                Err(ref e) if e == &Error::end_of_input() => break,
                Err(e) => panic!("Parse error at {}: {}", s.position(), e),
            }
        }
        return r;
    }
    fn tok_typ(s: &str) -> Vec<Kind> {
        let mut r = Vec::new();
        let mut s = TokenStream::new(s);
        loop {
            match s.uncons() {
                Ok(x) => r.push(x.kind),
                Err(ref e) if e == &Error::end_of_input() => break,
                Err(e) => panic!("Parse error at {}: {}", s.position(), e),
            }
        }
        return r;
    }

    #[test]
    fn comments_and_spaces() {
        assert_eq!(tok_str("# hello { world }"), &[] as &[&str]);
        assert_eq!(tok_str("# x\n\t\t# y"), &[] as &[&str]);
        assert_eq!(tok_str("\n \n"), &[] as &[&str]);
    }

    #[test]
    fn simple() {
        assert_eq!(tok_str("pid /run/nginx.pid;"),
                   ["pid", "/run/nginx.pid", ";"]);
        assert_eq!(tok_typ("pid /run/nginx.pid;"),
                   [String, String, Semicolon]);
        assert_eq!(tok_str("a { b }"), ["a", "{", "b", "}"]);
        assert_eq!(tok_typ("a { b }"), [String, BlockStart, String, BlockEnd]);
    }
    #[test]
    fn vars() {
        assert_eq!(tok_str("proxy_pass http://$x;"),
                   ["proxy_pass", "http://$x", ";"]);
        assert_eq!(tok_typ("proxy_pass http://$x;"),
                   [String, String, Semicolon]);
        assert_eq!(tok_str("proxy_pass http://${a b};"),
                   ["proxy_pass", "http://${a b}", ";"]);
        assert_eq!(tok_typ("proxy_pass http://${a b};"),
                   [String, String, Semicolon]);
    }
}
