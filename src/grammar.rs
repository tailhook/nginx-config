use combine::{eof, many, many1, Parser};
use combine::{choice, position};
use combine::combinator::{opaque, no_partial, FnOpaque};
use combine::error::StreamError;
use combine::easy::Error;

use ast::{self, Main, Directive, Item};
use error::ParseError;
use helpers::{semi, ident, text, string};
use position::Pos;
use tokenizer::{TokenStream, Token};
use value::Value;

use access;
use core;
use gzip;
use headers;
use proxy;
use rewrite;
use log;
use real_ip;


pub enum Code {
    Redirect(u32),
    Normal(u32),
}

pub fn bool<'a>() -> impl Parser<Output=bool, Input=TokenStream<'a>> {
    choice((
        ident("on").map(|_| true),
        ident("off").map(|_| false),
    ))
}

pub fn value<'a>() -> impl Parser<Output=Value, Input=TokenStream<'a>> {
    (position(), string())
    .and_then(|(p, v)| Value::parse(p, v))
}

pub fn worker_processes<'a>()
    -> impl Parser<Output=Item, Input=TokenStream<'a>>
{
    use ast::WorkerProcesses;
    ident("worker_processes")
    .with(choice((
        ident("auto").map(|_| WorkerProcesses::Auto),
        string().and_then(|s| s.value.parse().map(WorkerProcesses::Exact)),
    )))
    .skip(semi())
    .map(Item::WorkerProcesses)
}

pub fn server_name<'a>() -> impl Parser<Output=Item, Input=TokenStream<'a>> {
    use ast::ServerName::*;
    ident("server_name")
    .with(many1(
        string().map(|t| {
            if t.value.starts_with("~") {
                Regex(t.value[1..].to_string())
            } else if t.value.starts_with("*.") {
                StarSuffix(t.value[2..].to_string())
            } else if t.value.ends_with(".*") {
                StarPrefix(t.value[..t.value.len()-2].to_string())
            } else if t.value.starts_with(".") {
                Suffix(t.value[1..].to_string())
            } else {
                Exact(t.value.to_string())
            }
        })
    ))
    .skip(semi())
    .map(Item::ServerName)
}


pub fn map<'a>() -> impl Parser<Output=Item, Input=TokenStream<'a>> {
    use tokenizer::Kind::{BlockStart, BlockEnd};
    use helpers::kind;
    enum Tok {
        Hostnames,
        Volatile,
        Pattern(String, Value),
        Default(Value),
        Include(String),
    }
    ident("map")
    .with(value())
    .and(string().and_then(|t| {
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
    .skip(kind(BlockStart))
    .and(many(choice((
        ident("hostnames").map(|_| Tok::Hostnames),
        ident("volatile").map(|_| Tok::Volatile),
        ident("default").with(value()).map(|v| Tok::Default(v)),
        ident("include").with(raw()).map(|v| Tok::Include(v)),
        raw().and(value()).map(|(s, v)| Tok::Pattern(s, v)),
    )).skip(semi())))
    .skip(kind(BlockEnd))
    .map(|((expression, variable), vec): ((_, _), Vec<Tok>)| {
        let mut res = ::ast::Map {
            variable, expression,
            default: None,
            hostnames: false,
            volatile: false,
            includes: Vec::new(),
            patterns: Vec::new(),
        };
        for val in vec {
            match val {
                Tok::Hostnames => res.hostnames = true,
                Tok::Volatile => res.volatile = true,
                Tok::Default(v) => res.default = Some(v),
                Tok::Include(path) => res.includes.push(path),
                Tok::Pattern(x, targ) => {
                    use ast::MapPattern::*;
                    let mut s = &x[..];
                    if s.starts_with('~') {
                        res.patterns.push((Regex(s[1..].to_string()), targ));
                        continue;
                    } else if s.starts_with('\\') {
                        s = &s[1..];
                    }
                    let pat = if res.hostnames {
                        if s.starts_with("*.") {
                            StarSuffix(s[2..].to_string())
                        } else if s.ends_with(".*") {
                            StarPrefix(s[..s.len()-2].to_string())
                        } else if s.starts_with(".") {
                            Suffix(s[1..].to_string())
                        } else {
                            Exact(s.to_string())
                        }
                    } else {
                        Exact(s.to_string())
                    };
                    res.patterns.push((pat, targ));
                }
            }
        }
        Item::Map(res)
    })
}

pub fn block<'a>()
    -> FnOpaque<TokenStream<'a>, ((Pos, Pos), Vec<Directive>)>
{
    use tokenizer::Kind::{BlockStart, BlockEnd};
    use helpers::kind;
    opaque(|f| {
        f(&mut no_partial((
                position(),
                kind(BlockStart)
                    .with(many(directive()))
                    .skip(kind(BlockEnd)),
                position(),
        ))
        .map(|(s, dirs, e)| ((s, e), dirs)))
    })
}

// A string that forbids variables
pub fn raw<'a>() -> impl Parser<Output=String, Input=TokenStream<'a>> {
    // TODO(tailhook) unquote single and double quotes
    // error on variables?
    string().and_then(|t| Ok::<_, Error<_, _>>(t.value.to_string()))
}

pub fn location<'a>() -> impl Parser<Output=Item, Input=TokenStream<'a>> {
    use ast::LocationPattern::*;
    ident("location").with(choice((
        text("=").with(raw().map(Exact)),
        text("^~").with(raw().map(FinalPrefix)),
        text("~").with(raw().map(Regex)),
        text("~*").with(raw().map(RegexInsensitive)),
        raw()
            .map(|v| if v.starts_with('*') {
                Named(v)
            } else {
                Prefix(v)
            }),
    ))).and(block())
    .map(|(pattern, (position, directives))| {
        Item::Location(ast::Location { pattern, position, directives })
    })
}

impl Code {
    pub fn parse<'x, 'y>(code_str: &'x str)
        -> Result<Code, Error<Token<'y>, Token<'y>>>
    {
        let code = code_str.parse::<u32>()?;
        match code {
            301 | 302 | 303 | 307 | 308 => Ok(Code::Redirect(code)),
            200..=599 => Ok(Code::Normal(code)),
            _ => return Err(Error::unexpected_message(
                format!("invalid response code {}", code))),
        }
    }
    pub fn as_code(&self) -> u32 {
        match *self {
            Code::Redirect(code) => code,
            Code::Normal(code) => code,
        }
    }
}


pub fn try_files<'a>() -> impl Parser<Output=Item, Input=TokenStream<'a>> {
    use ast::TryFilesLastOption::*;
    use ast::Item::TryFiles;
    use value::Item::*;

    ident("try_files")
    .with(many1(value()))
    .skip(semi())
    .and_then(|mut v: Vec<_>| -> Result<_, Error<_, _>> {
        let last = v.pop().unwrap();
        let last = match &last.data[..] {
            [Literal(x)] if x.starts_with("=") => {
                Code(self::Code::parse(&x[1..])?.as_code())
            }
            [Literal(x)] if x.starts_with("@") => {
                NamedLocation(x[1..].to_string())
            }
            _ => Uri(last.clone()),
        };
        Ok(TryFiles(::ast::TryFiles {
            options: v,
            last_option: last,
        }))
    })
}


pub fn openresty<'a>() -> impl Parser<Output=Item, Input=TokenStream<'a>> {
    use ast::Item::*;
    choice((
        ident("rewrite_by_lua_file").with(value()).skip(semi())
            .map(Item::RewriteByLuaFile),
        ident("balancer_by_lua_file").with(value()).skip(semi())
            .map(BalancerByLuaFile),
        ident("access_by_lua_file").with(value()).skip(semi())
            .map(AccessByLuaFile),
        ident("header_filter_by_lua_file").with(value()).skip(semi())
            .map(HeaderFilterByLuaFile),
        ident("content_by_lua_file").with(value()).skip(semi())
            .map(ContentByLuaFile),
        ident("body_filter_by_lua_file").with(value()).skip(semi())
            .map(BodyFilterByLuaFile),
        ident("log_by_lua_file").with(value()).skip(semi())
            .map(LogByLuaFile),
        ident("lua_need_request_body").with(value()).skip(semi())
            .map(LuaNeedRequestBody),
        ident("ssl_certificate_by_lua_file").with(value()).skip(semi())
            .map(SslCertificateByLuaFile),
        ident("ssl_session_fetch_by_lua_file").with(value()).skip(semi())
            .map(SslSessionFetchByLuaFile),
        ident("ssl_session_store_by_lua_file").with(value()).skip(semi())
            .map(SslSessionStoreByLuaFile),
    ))
}

pub fn directive<'a>() -> impl Parser<Output=Directive, Input=TokenStream<'a>>
{
    position()
    .and(choice((
        ident("daemon").with(bool()).skip(semi())
            .map(Item::Daemon),
        ident("master_process").with(bool()).skip(semi())
            .map(Item::MasterProcess),
        worker_processes(),
        ident("http").with(block())
            .map(|(position, directives)| ast::Http { position, directives })
            .map(Item::Http),
        ident("server").with(block())
            .map(|(position, directives)| ast::Server { position, directives })
            .map(Item::Server),
        rewrite::directives(),
        try_files(),
        ident("include").with(value()).skip(semi()).map(Item::Include),
        ident("ssl_certificate").with(value()).skip(semi())
            .map(Item::SslCertificate),
        ident("ssl_certificate_key").with(value()).skip(semi())
            .map(Item::SslCertificateKey),
        location(),
        headers::directives(),
        server_name(),
        map(),
        ident("client_max_body_size").with(value()).skip(semi())
            .map(Item::ClientMaxBodySize),
        proxy::directives(),
        gzip::directives(),
        core::directives(),
        access::directives(),
        log::directives(),
        real_ip::directives(),
        openresty(),
        // it's own module
        ident("empty_gif").skip(semi()).map(|_| Item::EmptyGif),
        ident("index").with(many(value())).skip(semi())
            .map(Item::Index),
    )))
    .map(|(pos, dir)| Directive {
        position: pos,
        item: dir,
    })
}


/// Parses a piece of config in "main" context (i.e. top-level)
///
/// Currently, this is the same as parse_directives (except wraps everyting
/// to a `Main` struct), but we expect to
/// add validation/context checks in this function.
pub fn parse_main(s: &str) -> Result<Main, ParseError> {
    parse_directives(s).map(|directives| Main { directives })
}

/// Parses a piece of config from arbitrary context
///
/// This implies no validation of what context directives belong to.
pub fn parse_directives(s: &str) -> Result<Vec<Directive>, ParseError> {
    let mut tokens = TokenStream::new(s);
    let (doc, _) = many1(directive())
        .skip(eof())
        .parse_stream(&mut tokens)
        .map_err(|e| e.into_inner().error)?;
    Ok(doc)
}
