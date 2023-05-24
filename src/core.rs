use std::path::PathBuf;

use combine::{many, many1, Parser};
use combine::{choice, optional};
use combine::error::StreamError;
use combine::easy::Error;

use crate::ast::{self, Item};
use crate::grammar::{value, bool, block, Code};
use crate::helpers::{semi, ident, string, prefix};
use crate::tokenizer::{TokenStream, Token};
use crate::value::Value;


fn error_page<'a>()
    -> impl Parser<Output=Item, Input=TokenStream<'a>>
{
    use crate::ast::ErrorPageResponse;
    use crate::value::Item::*;

    fn lit<'a, 'x>(val: &'a Value) -> Result<&'a str, Error<Token<'x>, Token<'x>>> {
        if val.data.is_empty() {
            return Err(Error::unexpected_message(
                "empty error codes are not supported"));
        }
        if val.data.len() > 1 {
            return Err(Error::unexpected_message(
                "only last argument of error_codes \
                can contain variables"));
        }
        match val.data[0] {
            Literal(ref x) => return Ok(x),
            _ => return Err(Error::unexpected_message(
                "only last argument of error_codes \
                can contain variables")),
        }
    }

    let is_eq = |val: &Value| -> Result<bool, Error<_, _>> {
        Ok(lit(val)?.starts_with('='))
    };

    ident("error_page")
    .with(many(value()))
    .and_then(move |mut v: Vec<_>| {
        if v.is_empty() {
            return Err(Error::unexpected_message(
                "error_page directive must not be empty"));
        }
        let uri = v.pop().unwrap();

        let response_code = if v.last().is_some() && is_eq(v.last().unwrap())? {
            let dest = v.pop().unwrap();
            let dest = lit(&dest)?;
            if dest == "=" {
                ErrorPageResponse::Keep
            } else {
                match Code::parse(&dest[1..])? {
                    Code::Redirect(code) => ErrorPageResponse::Redirect(code),
                    Code::Normal(code) => ErrorPageResponse::Replace(code),
                }
            }
        } else {
            ErrorPageResponse::Target
        };
        let mut codes = Vec::new();
        for code in v {
            codes.push(Code::parse(lit(&code)?)?.as_code());
        }

        Ok(Item::ErrorPage(ast::ErrorPage {
            codes,
            response_code,
            uri,
        }))
    })
    .skip(semi())
}

enum ListenParts {
    DefaultServer,
    Ssl,
    Ext(ast::HttpExt),
    ProxyProtocol,
    SetFib(i32),
    FastOpen(u32),
    Backlog(i32),
    RcvBuf(u64),
    SndBuf(u64),
    Deferred,
    Bind,
    Ipv6Only(bool),
    ReusePort,
}

fn listen<'a>()
    -> impl Parser<Output=Item, Input=TokenStream<'a>>
{
    use ast::{Address, Listen, HttpExt};
    use self::ListenParts::*;

    ident("listen")
    .with(string().and_then(|s| -> Result<_, Error<_, _>> {
        let v = if s.value.starts_with("unix:") {
            Address::Unix(PathBuf::from(&s.value[6..]))
        } else if s.value.starts_with("*:") {
            Address::StarPort(s.value[2..].parse()?)
        } else {
            s.value.parse().map(Address::Port)
            .or_else(|_| s.value.parse().map(Address::Ip))?
        };
        Ok(v)
    }))
    .and(many::<Vec<_>, _>(choice((
        ident("default_server").map(|_| DefaultServer),
        ident("ssl").map(|_| Ssl),
        ident("http2").map(|_| Ext(HttpExt::Http2)),
        ident("spdy").map(|_| Ext(HttpExt::Spdy)),
        ident("proxy_protocol").map(|_| ProxyProtocol),
        prefix("setfib=").and_then(|val| val.parse().map(SetFib)),
        prefix("fastopen=").and_then(|val| val.parse().map(FastOpen)),
        prefix("backlog=").and_then(|val| val.parse().map(Backlog)),
        prefix("rcvbuf=").and_then(|val| val.parse().map(RcvBuf)),
        prefix("sndbuf=").and_then(|val| val.parse().map(SndBuf)),
        ident("deferred").map(|_| Deferred),
        ident("bind").map(|_| Bind),
        prefix("ipv6only=").and_then(|val| Ok(Ipv6Only(match val {
            "on" => true,
            "off" => false,
            _ => return Err(Error::unexpected_message("only on/off supported")),
        }))),
        ident("reuseport").map(|_| ReusePort),
    ))))
    .map(|(addr, items)| {
        let mut lst = Listen::new(addr);
        for item in items {
            match item {
                DefaultServer => lst.default_server = true,
                Ssl => lst.ssl = true,
                Ext(ext) => lst.ext = Some(ext),
                ProxyProtocol => lst.proxy_protocol = true,
                SetFib(v) => lst.setfib = Some(v),
                FastOpen(v) => lst.fastopen = Some(v),
                Backlog(v) => lst.backlog = Some(v),
                RcvBuf(v) => lst.rcvbuf = Some(v),
                SndBuf(v) => lst.sndbuf = Some(v),
                Deferred => lst.deferred = true,
                Bind => lst.bind = true,
                Ipv6Only(v) => lst.ipv6only = Some(v),
                ReusePort => lst.reuseport = true,
            }
        }
        return lst;
    })
    .skip(semi())
    .map(Item::Listen)
}

fn limit_except<'a>()
    -> impl Parser<Output=Item, Input=TokenStream<'a>>
{
    ident("limit_except")
    .with(many1(string().map(|x| x.value.to_string())))
    .and(block())
    .map(|(methods, (position, directives))| {
        Item::LimitExcept(ast::LimitExcept { methods, position, directives })
    })
}

pub fn directives<'a>()
    -> impl Parser<Output=Item, Input=TokenStream<'a>>
{
    choice((
        error_page(),
        listen(),
        limit_except(),
        ident("root").with(value()).skip(semi()).map(Item::Root),
        ident("alias").with(value()).skip(semi()).map(Item::Alias),
        ident("default_type").with(value()).skip(semi())
            .map(Item::DefaultType),
        ident("internal").skip(semi()).map(|_| Item::Internal),
        ident("etag").with(bool()).skip(semi()).map(Item::Etag),
        ident("server_tokens").with(value()).skip(semi())
            .map(Item::ServerTokens),
        ident("recursive_error_pages").with(bool()).skip(semi())
            .map(Item::RecursiveErrorPages),
        ident("chunked_transfer_encoding").with(bool()).skip(semi())
            .map(Item::ChunkedTransferEncoding),
        ident("keepalive_timeout")
            .with(value())
            .and(optional(value()))
            .map(|(timeo, htimeo)| Item::KeepaliveTimeout(timeo, htimeo))
            .skip(semi()),
        ident("error_log").with(value())
            .and(optional(string().and_then(|t| {
                use ast::ErrorLevel::*;
                match t.value {
                    "debug" => Ok(Debug),
                    "info" => Ok(Info),
                    "notice" => Ok(Notice),
                    "warn" => Ok(Warn),
                    "error" => Ok(Error),
                    "crit" => Ok(Crit),
                    "alert" => Ok(Alert),
                    "emerg" => Ok(Emerg),
                    _ => Err(::combine::easy::Error::unexpected_message(
                            "invalid log level")),
                }
            })))
            .skip(semi())
            .map(|(file, level)| Item::ErrorLog { file, level })
    ))
}
