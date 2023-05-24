use combine::{many, Parser};
use combine::{choice, optional, position};
use combine::error::StreamError;
use combine::easy::Error;

use crate::ast::{self, Item};
use crate::grammar::{value};
use crate::helpers::{semi, ident, string};
use crate::tokenizer::{TokenStream};
use crate::value::Value;


fn access_log<'a>()
    -> impl Parser<Output=Item, Input=TokenStream<'a>>
{
    enum I {
        If(Value),
        Gzip(Option<u8>),
        Flush(String),
        Buffer(String),
    }

    ident("access_log")
    .with(choice((
        ident("off").map(|_| ast::AccessLog::Off),
        value().and(optional(
            string()
            .and(many::<Vec<_>, _>(
                (position(), string()).and_then(|(pos, s)| {
                    if s.value.starts_with("if=") {
                        Ok(I::If(Value::parse_str(pos, &s.value[3..])?))
                    } else if s.value == "gzip" {
                        Ok(I::Gzip(None))
                    } else if s.value.starts_with("gzip=") {
                        Ok(I::Gzip(Some(s.value[5..].parse()?)))
                    } else if s.value.starts_with("buffer=") {
                        Ok(I::Buffer(s.value[7..].to_string()))
                    } else if s.value.starts_with("flush=") {
                        Ok(I::Flush(s.value[6..].to_string()))
                    } else {
                        Err(Error::unexpected_message(
                            format!("bad access_log param {:?}", s.value)))
                    }
                })))
        )).map(|(path, params)| {
            let mut res = ast::AccessLogOptions {
                path,
                format: None,
                buffer: None,
                gzip: None,
                flush: None,
                condition: None,
            };
            if let Some((format, params)) = params {
                res.format = Some(format.value.to_string());
                for item in params {
                    match item {
                        I::If(val) => res.condition = Some(val),
                        I::Buffer(buf) => res.buffer = Some(buf),
                        I::Gzip(gzip) => res.gzip = Some(gzip),
                        I::Flush(flush) => res.flush = Some(flush),
                    }
                }
            }
            ast::AccessLog::On(res)
        })
    )))
    .skip(semi())
    .map(Item::AccessLog)
}

pub fn directives<'a>()
    -> impl Parser<Output=Item, Input=TokenStream<'a>>
{
    choice((
        access_log(),
    ))
}
