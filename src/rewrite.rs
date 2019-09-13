use combine::{Parser};
use combine::{choice, optional, many1, position};
use combine::error::StreamError;
use combine::easy::Error;

use ast::{self, Item};
use grammar::{value, block, Code};
use helpers::{semi, ident, string};
use position::Pos;
use tokenizer::{TokenStream, Token};
use value::{Value};


fn rewrite<'a>()
    -> impl Parser<Output=Item, Input=TokenStream<'a>>
{
    use ast::RewriteFlag::*;
    use ast::Item::Rewrite;

    ident("rewrite")
    .with(string())
    .and(value())
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

fn set<'a>()
    -> impl Parser<Output=Item, Input=TokenStream<'a>>
{
    ident("set")
    .with(string().and_then(|t| {
        let ch1 = t.value.chars().nth(0).unwrap_or(' ');
        let ch2 = t.value.chars().nth(1).unwrap_or(' ');
        if ch1 == '$' && matches!(ch2, 'a'..='z' | 'A'..='Z' | '_') &&
            t.value[2..].chars()
            .all(|x| matches!(x, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_'))
        {
            Ok(t.value[1..].to_string())
        } else {
            Err(Error::unexpected_message("invalid variable"))
        }
    }))
    .and(value())
    .skip(semi())
    .map(|(variable, value)| Item::Set { variable, value })
}

fn return_directive<'a>()
    -> impl Parser<Output=Item, Input=TokenStream<'a>>
{
    use ast::Return::*;
    use value::Item::*;

    fn lit<'a, 'x>(val: &'a Value) -> Result<&'a str, Error<Token<'x>, Token<'x>>> {
        if val.data.is_empty() {
            return Err(Error::unexpected_message(
                "empty return codes are not supported"));
        }
        if val.data.len() > 1 {
            return Err(Error::unexpected_message(
                "return code can't contain variables"));
        }
        match val.data[0] {
            Literal(ref x) => return Ok(x),
            _ => return Err(Error::unexpected_message(
                "return code can't contain variables")),
        }
    }

    ident("return")
    .with(value().and(optional(value())))
    .and_then(|(a, b)| -> Result<_, Error<_, _>> {
        if let Some(target) = b {
            match Code::parse(lit(&a)?)? {
                Code::Redirect(code)
                => Ok(Redirect { code: Some(code), url: target }),
                Code::Normal(code)
                => Ok(Text { code: code, text: Some(target) }),
            }
        } else {
            match a.data.get(0) {
                Some(Literal(x))
                if x.starts_with("https://") || x.starts_with("http://")
                => Ok(Redirect { code: None, url: a.clone()}),
                Some(Variable(v)) if v == "scheme"
                => Ok(Redirect { code: None, url: a.clone()}),
                _ => {
                    match Code::parse(lit(&a)?)? {
                        Code::Redirect(_)
                        => return Err(Error::unexpected_message(
                            "return with redirect code must have \
                             destination URI")),
                        Code::Normal(code)
                        => Ok(Text { code: code, text: None }),
                    }

                }
            }
        }
    })
    .map(Item::Return)
    .skip(semi())
}

fn strip_open_paren<'a>(v: &'_ mut Vec<&'a str>)
    -> Result<(), Error<Token<'a>, Token<'a>>>
{
    match v.get_mut(0) {
        Some(s) => {
            if s.starts_with('(') {
                if &s[..] != "(" {
                    *s = &s[1..];
                    return Ok(())
                }
            } else {
                return Err(Error::unexpected_message("missing parenthesis"));
            }
        }
        _ => return Err(Error::unexpected_message("missing parenthesis")),
    }
    v.remove(0);
    Ok(())
}

fn strip_close_paren<'a>(v: &'_ mut Vec<&'a str>)
    -> Result<(), Error<Token<'a>, Token<'a>>>
{
    match v.last_mut() {
        Some(s) => {
            if s.ends_with(')') {
                if &s[..] != ")" {
                    let new_len = s.len()-1;
                    *s = &s[..new_len];
                    return Ok(());
                }
            } else {
                return Err(Error::unexpected_message("missing parenthesis"));
            }
        }
        _ => return Err(Error::unexpected_message("missing parenthesis")),
    }
    v.pop();
    Ok(())
}

fn parse_unary<'a>(mut v: Vec<&str>, position: Pos)
    -> Result<ast::IfCondition, Error<Token<'a>, Token<'a>>>
{
    use ast::IfCondition::*;

    let oper = v.remove(0);
    let right = Value::parse_str(position, v.remove(0))?;
    if v.len() > 0 {
        return Err(Error::unexpected_message("extra argument to condition"));
    }
    match oper {
        "-d" => return Ok(DirExists(right)),
        "!-d" => return Ok(DirNotExists(right)),
        "-f" => return Ok(FileExists(right)),
        "!-f" => return Ok(FileNotExists(right)),
        "-x" => return Ok(Executable(right)),
        "!-x" => return Ok(NotExecutable(right)),
        "-e" => return Ok(Exists(right)),
        "!-e" => return Ok(NotExists(right)),
        _ => return Err(Error::unexpected_message("missing parenthesis")),
    }
}

fn parse_binary<'a>(mut v: Vec<&str>, position: Pos)
    -> Result<ast::IfCondition, Error<Token<'a>, Token<'a>>>
{
    use ast::IfCondition::*;

    let left = Value::parse_str(position, v.remove(0))?;
    if v.len() == 0 {
        return Ok(NonEmpty(left));
    }
    let oper = v.remove(0);
    let right = match &v[..] {
        [x] => x.to_string(),
        _ => return Err(Error::unexpected_message(
                "you can only compare against a single literal")),
    };
    match oper {
        "=" => return Ok(Eq(left, right)),
        "!=" => return Ok(Neq(left, right)),
        "~" => return Ok(RegEq(left, right, true)),
        "!~" => return Ok(RegNeq(left, right, true)),
        "~*" => return Ok(RegEq(left, right, false)),
        "!~*" => return Ok(RegNeq(left, right, false)),
        _ => return Err(Error::unexpected_message("missing parenthesis")),
    }
}

fn if_directive<'a>()
    -> impl Parser<Output=Item, Input=TokenStream<'a>>
{
    ident("if")
    .with(position())
    .and(many1(string()))
    .and_then(|(pos, v): (_, Vec<_>)| -> Result<_, Error<_, _>> {
        let mut v = v.iter().map(|t| t.value).collect();
        strip_open_paren(&mut v)?;
        strip_close_paren(&mut v)?;
        let binary = match v.get(0) {
            Some(x) if x.starts_with('$') => true,
            Some(_) => false,
            None => return Err(Error::unexpected_message(
                "missing parenthesis")),
        };
        if binary {
            parse_binary(v, pos)
        } else {
            parse_unary(v, pos)
        }
    })
    .and(block())
    .map(|(condition, (position, directives))| {
        ast::If { position, condition, directives }
    })
    .map(Item::If)
}

pub fn directives<'a>()
    -> impl Parser<Output=Item, Input=TokenStream<'a>>
{
    choice((
        rewrite(),
        set(),
        return_directive(),
        if_directive(),
    ))
}
