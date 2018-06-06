use combine::{parser, Parser};
use combine::{choice, optional};
use combine::error::StreamError;
use combine::easy::Error;

use ast::{self, Item};
use grammar::{value, Code};
use helpers::{semi, ident, string};
use tokenizer::{TokenStream, Token};
use value::Value;


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

fn set<'a>()
    -> impl Parser<Output=Item, Input=TokenStream<'a>>
{
    ident("set")
    .with(string().and_then(|t| {
        let ch1 = t.value.chars().nth(0).unwrap_or(' ');
        let ch2 = t.value.chars().nth(1).unwrap_or(' ');
        if ch1 == '$' && matches!(ch2, 'a'...'z' | 'A'...'Z' | '_') &&
            t.value[2..].chars()
            .all(|x| matches!(x, 'a'...'z' | 'A'...'Z' | '0'...'9' | '_'))
        {
            Ok(t.value[1..].to_string())
        } else {
            Err(Error::unexpected_message("invalid variable"))
        }
    }))
    .and(parser(value))
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
    .with(parser(value).and(optional(parser(value))))
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

pub fn directives<'a>()
    -> impl Parser<Output=Item, Input=TokenStream<'a>>
{
    choice((
        rewrite(),
        set(),
        return_directive(),
    ))
}
