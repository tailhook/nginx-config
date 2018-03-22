use combine::easy::Error;
use combine::error::StreamError;

use format::{Displayable, Formatter};
use position::Pos;
use tokenizer::Token;

/// Generic string value
///
/// It may consist of strings and variable references
///
/// Some string parts might originally be escaped or quoted. We get rid of
/// quotes when parsing
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Value {
    position: Pos,
    data: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Item {
    Literal(String),
    Variable(String),
}


impl Value {
    pub(crate) fn parse<'a>(pos: Pos, tok: Token<'a>)
        -> Result<Value, Error<Token<'a>, Token<'a>>>
    {
        use self::Item::*;
        let mut buf = Vec::new();
        let ref val = tok.value;
        let mut chiter = val.char_indices().peekable();
        let mut prev_char = ' ';  // any having no special meaning
        let mut cur_slice = 0;
        // TODO(unquote) single and double quotes
        while let Some((idx, cur_char)) = chiter.next() {
            match cur_char {
                _ if prev_char == '\\' => {
                    prev_char = ' ';
                    continue;
                }
                '$' => {
                    let vstart = idx + 1;
                    if idx != cur_slice {
                        buf.push(Literal(val[cur_slice..idx].to_string()));
                    }
                    let fchar = chiter.next().map(|(_, c)| c)
                        .ok_or_else(|| Error::unexpected_message(
                            "bare $ in expression"))?;
                    match fchar {
                        '{' => {
                            unimplemented!();
                        }
                        'a'...'z' | 'A'...'Z' | '_' => {
                            while let Some(&(_, c)) = chiter.peek() {
                                match c {
                                    'a'...'z' | 'A'...'Z' | '_' | '0'...'9'
                                    => chiter.next(),
                                    _ => break,
                                };
                            }
                            let now = chiter.peek().map(|&(idx, _)| idx)
                                .unwrap_or(val.len());
                            buf.push(Variable(
                                val[vstart..now].to_string()));
                            cur_slice = now;
                        }
                        _ => {
                            return Err(Error::unexpected_message(
                                format!("variable name starts with \
                                    bad char {:?}", fchar)));
                        }
                    }
                }
                _ => {}
            }
            prev_char = cur_char;
        }
        if cur_slice != val.len() {
            buf.push(Literal(val[cur_slice..].to_string()));
        }
        Ok(Value {
            position: pos,
            data: buf,
        })
    }
}

impl Value {
    fn has_specials(&self) -> bool {
        use self::Item::*;
        for item in &self.data {
            match *item {
                Literal(ref x) => {
                    for c in x.chars() {
                        match c {
                            ' ' | ';' | '\r' | '\n' | '\t' => {
                                return true;
                            }
                            _ => {}
                        }
                    }
                }
                Variable(_) => {}
            }
        }
        return false;
    }
}

impl Displayable for Value {
    fn display(&self, f: &mut Formatter) {
        use self::Item::*;
        if self.has_specials() {
            unimplemented!();
        } else {
            for item in &self.data {
                match *item {
                    Literal(ref v) => f.write(v),
                    Variable(ref v) => { f.write("$"); f.write(v); }
                }
            }
        }
    }
}
