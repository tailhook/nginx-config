use std::mem;
use std::str::FromStr;

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
    pub(crate) data: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Item {
    Literal(String),
    Variable(String),
}


impl Value {
    pub(crate) fn parse<'a>(position: Pos, tok: Token<'a>)
        -> Result<Value, Error<Token<'a>, Token<'a>>>
    {
        Value::parse_str(position, tok.value)
    }
    pub(crate) fn parse_str<'a>(position: Pos, token: &str)
        -> Result<Value, Error<Token<'a>, Token<'a>>>
    {
        let data = if token.starts_with('"') {
            Value::scan_quoted('"', token)?
        } else if token.starts_with("'") {
            Value::scan_quoted('\'', token)?
        } else {
            Value::scan_raw(token)?
        };
        Ok(Value { position, data })
    }

    fn scan_raw<'a>(value: &str)
        -> Result<Vec<Item>, Error<Token<'a>, Token<'a>>>
    {
        use self::Item::*;
        let mut buf = Vec::new();
        let mut chiter = value.char_indices().peekable();
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
                        buf.push(Literal(value[cur_slice..idx].to_string()));
                    }
                    let fchar = chiter.next().map(|(_, c)| c)
                        .ok_or_else(|| Error::unexpected_message(
                            "bare $ in expression"))?;
                    match fchar {
                        '{' => {
                            while let Some(&(_, c)) = chiter.peek() {
                                match c {
                                    'a'..='z' | 'A'..='Z' | '_' | '0'..='9'
                                    => chiter.next(),
                                    '}' => break,
                                    _ => {
                                        return Err(Error::expected("}".into()));
                                    }
                                };
                            }
                            let now = chiter.peek().map(|&(idx, _)| idx)
                                .unwrap();
                            buf.push(Variable(
                                value[vstart+1..now].to_string()));
                            cur_slice = now+1;
                        }
                        'a'..='z' | 'A'..='Z' | '_' | '0'..='9' => {
                            while let Some(&(_, c)) = chiter.peek() {
                                match c {
                                    'a'..='z' | 'A'..='Z' | '_' | '0'..='9'
                                    => chiter.next(),
                                    _ => break,
                                };
                            }
                            let now = chiter.peek().map(|&(idx, _)| idx)
                                .unwrap_or(value.len());
                            buf.push(Variable(
                                value[vstart..now].to_string()));
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
        if cur_slice != value.len() {
            buf.push(Literal(value[cur_slice..].to_string()));
        }
        Ok(buf)
    }

    fn scan_quoted<'a>(quote: char, value: &str)
        -> Result<Vec<Item>, Error<Token<'a>, Token<'a>>>
    {
        use self::Item::*;
        let mut buf = Vec::new();
        let mut chiter = value.char_indices().peekable();
        chiter.next(); // skip quote
        let mut prev_char = ' ';  // any having no special meaning
        let mut cur_slice = String::new();
        while let Some((idx, cur_char)) = chiter.next() {
            match cur_char {
                _ if prev_char == '\\' => {
                    cur_slice.push(cur_char);
                    continue;
                }
                '"' | '\'' if cur_char == quote => {
                    if cur_slice.len() > 0 {
                        buf.push(Literal(cur_slice));
                    }
                    if idx + 1 != value.len() {
                        // TODO(tailhook) figure out maybe this is actually a
                        // tokenizer error, or maybe make this cryptic message
                        // better
                        return Err(Error::unexpected_message(
                            "quote closes prematurely"));
                    }
                    return Ok(buf);
                }
                '$' => {
                    let vstart = idx + 1;
                    if cur_slice.len() > 0 {
                        buf.push(Literal(
                            mem::replace(&mut cur_slice, String::new())));
                    }
                    let fchar = chiter.next().map(|(_, c)| c)
                        .ok_or_else(|| Error::unexpected_message(
                            "bare $ in expression"))?;
                    match fchar {
                        '{' => {
                            unimplemented!();
                        }
                        'a'..='z' | 'A'..='Z' | '_' | '0'..='9' => {
                            while let Some(&(_, c)) = chiter.peek() {
                                match c {
                                    'a'..='z' | 'A'..='Z' | '_' | '0'..='9'
                                    => chiter.next(),
                                    _ => break,
                                };
                            }
                            let now = chiter.peek().map(|&(idx, _)| idx)
                                .ok_or_else(|| {
                                    Error::unexpected_message("unclosed quote")
                                })?;
                            buf.push(Variable(
                                value[vstart..now].to_string()));
                        }
                        _ => {
                            return Err(Error::unexpected_message(
                                format!("variable name starts with \
                                    bad char {:?}", fchar)));
                        }
                    }
                }
                _ => cur_slice.push(cur_char),
            }
            prev_char = cur_char;
        }
        return Err(Error::unexpected_message("unclosed quote"));
    }
}

impl FromStr for Value {
    type Err = String;
    fn from_str(s: &str) -> Result<Value, String> {
        Value::parse_str(Pos { line: 0, column: 0 }, s)
        .map_err(|e| e.to_string())
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
                            ' ' | ';' | '\r' | '\n' | '\t' | '{' | '}' => {
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

    /// Replace variable references in this string with literal values
    pub fn replace_vars<'a, F, S>(&mut self, mut f: F)
        where F: FnMut(&str) -> Option<S>,
              S: AsRef<str> + Into<String> + 'a,
    {
        use self::Item::*;
        // TODO(tailhook) join literal blocks
        for item in &mut self.data {
            let new_value = match *item {
                Literal(..) => continue,
                Variable(ref name) => match f(name) {
                    Some(value) => value.into(),
                    None => continue,
                },
            };
            *item = Literal(new_value);
        }
    }
}

fn next_alphanum(data: &Vec<Item>, index: usize) -> bool {
    use self::Item::*;
    data.get(index+1).and_then(|item| {
        match item {
            Literal(s) => Some(s),
            Variable(_) => None,
        }
    }).and_then(|s| {
        s.chars().next().map(|c| c.is_alphanumeric())
    }).unwrap_or(false)
}

impl Displayable for Value {
    fn display(&self, f: &mut Formatter) {
        use self::Item::*;
        if self.data.is_empty() || self.has_specials() {
            f.write("\"");
            for (index, item) in self.data.iter().enumerate() {
                match *item {
                    // TODO(tailhook) escape special chars
                    Literal(ref v) => f.write(v),
                    Variable(ref v) if next_alphanum(&self.data, index) => {
                        f.write("${");
                        f.write(v);
                        f.write("}");
                    }
                    Variable(ref v) => {
                        f.write("$");
                        f.write(v);
                    }
                }
            }
            f.write("\"");
        } else {
            for (index, item) in self.data.iter().enumerate() {
                match *item {
                    Literal(ref v) => f.write(v),
                    Variable(ref v) if next_alphanum(&self.data, index) => {
                        f.write("${");
                        f.write(v);
                        f.write("}");
                    }
                    Variable(ref v) => {
                        f.write("$");
                        f.write(v);
                    }
                }
            }
        }
    }
}
