use combine::{eof, many, many1, ParseResult, parser, Parser};
use combine::{choice, position, optional};
use combine::error::StreamError;
use combine::easy::Error;

use ast::{self, Main, Directive, Item};
use core;
use error::ParseError;
use gzip;
use helpers::{semi, ident, text, string};
use position::Pos;
use proxy;
use tokenizer::{TokenStream, Token};
use value::Value;

pub enum Code {
    Redirect(u32),
    Normal(u32),
}

pub fn bool<'a>(input: &mut TokenStream<'a>)
    -> ParseResult<bool, TokenStream<'a>>
{
    choice((
        ident("on").map(|_| true),
        ident("off").map(|_| false),
    ))
    .parse_stream(input)
}

pub fn value<'a>(input: &mut TokenStream<'a>)
    -> ParseResult<Value, TokenStream<'a>>
{
    (position(), string())
    .and_then(|(p, v)| Value::parse(p, v))
    .parse_stream(input)
}

pub fn worker_processes<'a>(input: &mut TokenStream<'a>)
    -> ParseResult<Item, TokenStream<'a>>
{
    use ast::WorkerProcesses;
    ident("worker_processes")
    .with(choice((
        ident("auto").map(|_| WorkerProcesses::Auto),
        string().and_then(|s| s.value.parse().map(WorkerProcesses::Exact)),
    )))
    .skip(semi())
    .map(Item::WorkerProcesses)
    .parse_stream(input)
}

pub fn add_header<'a>(input: &mut TokenStream<'a>)
    -> ParseResult<Item, TokenStream<'a>>
{
    ident("add_header")
    .with((
        parser(value),
        parser(value),
        optional(ident("always").map(|_| ())),
    )).map(|(field, value, always)| {
        ast::AddHeader { field, value, always: always.is_some() }
    })
    .skip(semi())
    .map(Item::AddHeader)
    .parse_stream(input)
}

pub fn server_name<'a>(input: &mut TokenStream<'a>)
    -> ParseResult<Item, TokenStream<'a>>
{
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
    .parse_stream(input)
}

pub fn set<'a>(input: &mut TokenStream<'a>)
    -> ParseResult<Item, TokenStream<'a>>
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
    .parse_stream(input)
}

pub fn map<'a>(input: &mut TokenStream<'a>)
    -> ParseResult<Item, TokenStream<'a>>
{
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
    .with(parser(value))
    .and(string().and_then(|t| {
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
    .skip(kind(BlockStart))
    .and(many(choice((
        ident("hostnames").map(|_| Tok::Hostnames),
        ident("volatile").map(|_| Tok::Volatile),
        ident("default").with(parser(value)).map(|v| Tok::Default(v)),
        ident("include").with(parser(raw)).map(|v| Tok::Include(v)),
        parser(raw).and(parser(value)).map(|(s, v)| Tok::Pattern(s, v)),
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
    .parse_stream(input)
}

pub fn block<'a>(input: &mut TokenStream<'a>)
    -> ParseResult<((Pos, Pos), Vec<Directive>), TokenStream<'a>>
{
    use tokenizer::Kind::{BlockStart, BlockEnd};
    use helpers::kind;
    (
        position(),
        kind(BlockStart)
            .with(many(parser(directive)))
            .skip(kind(BlockEnd)),
        position(),
    )
    .map(|(s, dirs, e)| ((s, e), dirs))
    .parse_stream(input)
}

// A string that forbids variables
pub fn raw<'a>(input: &mut TokenStream<'a>)
    -> ParseResult<String, TokenStream<'a>>
{
    // TODO(tailhook) unquote single and double quotes
    // error on variables?
    string().and_then(|t| Ok::<_, Error<_, _>>(t.value.to_string()))
    .parse_stream(input)
}

pub fn location<'a>(input: &mut TokenStream<'a>)
    -> ParseResult<Item, TokenStream<'a>>
{
    use ast::LocationPattern::*;
    ident("location").with(choice((
        text("=").with(parser(raw).map(Exact)),
        text("^~").with(parser(raw).map(FinalPrefix)),
        text("~").with(parser(raw).map(Regex)),
        text("~*").with(parser(raw).map(RegexInsensitive)),
        parser(raw)
            .map(|v| if v.starts_with('*') {
                Named(v)
            } else {
                Prefix(v)
            }),
    ))).and(parser(block))
    .map(|(pattern, (position, directives))| {
        Item::Location(ast::Location { pattern, position, directives })
    })
    .parse_stream(input)
}

impl Code {
    pub fn parse<'x, 'y>(code_str: &'x str)
        -> Result<Code, Error<Token<'y>, Token<'y>>>
    {
        let code = code_str.parse::<u32>()?;
        match code {
            301 | 302 | 303 | 307 | 308 => Ok(Code::Redirect(code)),
            200...599 => Ok(Code::Normal(code)),
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

pub fn rewrite<'a>(input: &mut TokenStream<'a>)
    -> ParseResult<Item, TokenStream<'a>>
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
    .parse_stream(input)
}

pub fn try_files<'a>(input: &mut TokenStream<'a>)
    -> ParseResult<Item, TokenStream<'a>>
{
    use ast::TryFilesLastOption::*;
    use ast::Item::TryFiles;
    use value::Item::*;

    ident("try_files")
    .with(many1(parser(value)))
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
    .parse_stream(input)
}


pub fn return_directive<'a>(input: &mut TokenStream<'a>)
    -> ParseResult<Item, TokenStream<'a>>
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
    .parse_stream(input)
}


pub fn openresty<'a>(input: &mut TokenStream<'a>)
    -> ParseResult<Item, TokenStream<'a>>
{
    use ast::Item::*;
    choice((
        ident("rewrite_by_lua_file").with(parser(value)).skip(semi())
            .map(Item::RewriteByLuaFile),
        ident("balancer_by_lua_file").with(parser(value)).skip(semi())
            .map(BalancerByLuaFile),
        ident("access_by_lua_file").with(parser(value)).skip(semi())
            .map(AccessByLuaFile),
        ident("header_filter_by_lua_file").with(parser(value)).skip(semi())
            .map(HeaderFilterByLuaFile),
        ident("content_by_lua_file").with(parser(value)).skip(semi())
            .map(ContentByLuaFile),
        ident("body_filter_by_lua_file").with(parser(value)).skip(semi())
            .map(BodyFilterByLuaFile),
        ident("log_by_lua_file").with(parser(value)).skip(semi())
            .map(LogByLuaFile),
        ident("lua_need_request_body").with(parser(value)).skip(semi())
            .map(LuaNeedRequestBody),
        ident("ssl_certificate_by_lua_file").with(parser(value)).skip(semi())
            .map(SslCertificateByLuaFile),
        ident("ssl_session_fetch_by_lua_file").with(parser(value)).skip(semi())
            .map(SslSessionFetchByLuaFile),
        ident("ssl_session_store_by_lua_file").with(parser(value)).skip(semi())
            .map(SslSessionStoreByLuaFile),
    ))
    .parse_stream(input)
}

pub fn directive<'a>(input: &mut TokenStream<'a>)
    -> ParseResult<Directive, TokenStream<'a>>
{
    position()
    .and(choice((
        ident("daemon").with(parser(bool)).skip(semi())
            .map(Item::Daemon),
        ident("master_process").with(parser(bool)).skip(semi())
            .map(Item::MasterProcess),
        parser(worker_processes),
        ident("http").with(parser(block))
            .map(|(position, directives)| ast::Http { position, directives })
            .map(Item::Http),
        ident("server").with(parser(block))
            .map(|(position, directives)| ast::Server { position, directives })
            .map(Item::Server),
        parser(return_directive),
        parser(rewrite),
        parser(try_files),
        ident("include").with(parser(value)).skip(semi()).map(Item::Include),
        ident("ssl_certificate").with(parser(value)).skip(semi())
            .map(Item::SslCertificate),
        ident("ssl_certificate_key").with(parser(value)).skip(semi())
            .map(Item::SslCertificateKey),
        parser(location),
        parser(add_header),
        parser(server_name),
        parser(set),
        parser(map),
        ident("client_max_body_size").with(parser(value)).skip(semi())
            .map(Item::ClientMaxBodySize),
        parser(proxy::directives),
        parser(gzip::directives),
        core::directives(),
        parser(openresty),
    )))
    .map(|(pos, dir)| Directive {
        position: pos,
        item: dir,
    })
    .parse_stream(input)
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
    let (doc, _) = many1(parser(directive))
        .skip(eof())
        .parse_stream(&mut tokens)
        .map_err(|e| e.into_inner().error)?;
    Ok(doc)
}
