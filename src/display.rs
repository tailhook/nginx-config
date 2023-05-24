use crate::ast;
use std::fmt;
use crate::format::{Displayable, Formatter, Style};

use crate::value;

impl Displayable for ast::Main {
    fn display(&self, f: &mut Formatter) {
        for item in &self.directives {
            item.display(f);
        }
    }
}

impl Displayable for ast::Directive {
    fn display(&self, f: &mut Formatter) {
        self.item.display(f)
    }
}

fn simple_block<D: fmt::Display>(f: &mut Formatter, name: D,
    directives: &[ast::Directive])
{
    f.margin();
    f.indent();
    f.fmt(&format_args!("{} ", name));
    f.start_block();
    for dir in directives {
        dir.display(f);
    }
    f.end_block();
}

fn one_arg_dir(name: &str, val: &value::Value, f: &mut Formatter) {
    f.indent();
    f.write(name);
    f.write(" ");
    val.display(f);
    f.end();
}

impl Displayable for ast::Item {
    fn display(&self, f: &mut Formatter) {
        use ast::Item::*;
        match *self {
            | Daemon(opt)
            | MasterProcess(opt)
            | ProxyPassRequestHeaders(opt)
            | ProxyPassRequestBody(opt)
            | ProxyInterceptErrors(opt)
            | ProxyBuffering(opt)
            | Gzip(opt)
            | Etag(opt)
            | RecursiveErrorPages(opt)
            | ChunkedTransferEncoding(opt)
            | RealIpRecursive(opt)
            => {
                f.indent();
                f.write(self.directive_name());
                f.write(" ");
                f.write(if opt { "on" } else { "off" });
                f.end();
            }
            WorkerProcesses(ast::WorkerProcesses::Auto) => {
                f.indent();
                f.write("worker_processes auto");
                f.end();
            }
            WorkerProcesses(ast::WorkerProcesses::Exact(n)) => {
                f.indent();
                f.write("worker_processes ");
                f.fmt(&n);
                f.end();
            }
            Http(ref h) => {
                simple_block(f, "http", &h.directives);
            }
            Server(ref s) => {
                simple_block(f, "server", &s.directives);
            }
            Location(ast::Location { ref pattern, ref directives, .. }) => {
                simple_block(f,
                    format_args!("location {}", pattern),
                    &directives);
            }
            LimitExcept(ast::LimitExcept { ref methods, ref directives, .. })
            => {
                simple_block(f,
                    format_args!("limit_except {}", methods.join(" ")),
                    &directives);
            }
            Listen(ref lst) => {
                f.indent();
                lst.display(f);
            }
            ProxySetHeader { ref field, ref value } => {
                f.indent();
                f.write("proxy_set_header ");
                field.display(f);
                f.write(" ");
                value.display(f);
                f.end();
            }
            GzipStatic(opt) => {
                f.indent();
                f.write("gzip_static ");
                f.write(opt.as_str());
                f.end();
            }
            GzipProxied(ref opt) => {
                f.indent();
                f.write("gzip_proxied");
                for item in opt {
                    f.write(" ");
                    f.write(item.as_str());
                }
                f.end();
            }
            AddHeader(ref h) => {
                f.indent();
                f.write("add_header ");
                h.field.display(f);
                f.write(" ");
                h.value.display(f);
                if h.always {
                    f.write(" always");
                }
                f.end();
            }
            ServerName(ref items) => {
                use ast::ServerName::*;
                f.indent();
                f.write("server_name");
                for item in items {
                    match *item {
                        Exact(ref v)
                        => f.fmt(&format_args!(" {}", escape(&v))),
                        Suffix(ref v)
                        => f.fmt(&format_args!(" .{}", escape(&v))),
                        StarSuffix(ref v)
                        => f.fmt(&format_args!(" *.{}", escape(&v))),
                        StarPrefix(ref v)
                        => f.fmt(&format_args!(" {}.*", escape(&v))),
                        Regex(ref v)
                        => f.fmt(&format_args!(" ~{}", escape(&v))),
                    }
                }
                f.end();
            }
            Set { ref variable, ref value } => {
                f.indent();
                f.write("set $");
                f.write(variable); // TODO(tailhook) check syntax?
                f.write(" ");
                value.display(f);
                f.end();
            }
            Map(ref m) => {
                use ast::MapPattern::*;
                f.margin();
                f.indent();
                f.write("map ");
                m.expression.display(f);
                f.write(" $");
                f.write(&m.variable); // TODO(tailhook) check syntax?
                f.write(" ");
                f.start_block();
                if m.volatile {
                    f.indent();
                    f.write("volatile");
                    f.end();
                }
                if m.hostnames {
                    f.indent();
                    f.write("hostnames");
                    f.end();
                }
                if let Some(ref def) = m.default {
                    f.indent();
                    f.write("default ");
                    def.display(f);
                    f.end();
                }
                for inc in &m.includes {
                    f.indent();
                    f.write("include ");
                    f.write(escape(inc));
                    f.end();
                }
                for &(ref pat, ref value) in &m.patterns {
                    f.indent();
                    match *pat {
                        Exact(ref v) if matches!(&v[..],
                            | "volatile"
                            | "hostnames"
                            | "default"
                            | "include"
                        ) => f.fmt(&format_args!("\\{}", escape(&v))),
                        Exact(ref v)
                        => f.fmt(&format_args!("{}", escape(&v))),
                        Suffix(ref v)
                        => f.fmt(&format_args!(".{}", escape(&v))),
                        StarSuffix(ref v)
                        => f.fmt(&format_args!("*.{}", escape(&v))),
                        StarPrefix(ref v)
                        => f.fmt(&format_args!("{}.*", escape(&v))),
                        Regex(ref v)
                        => f.fmt(&format_args!("~{}", escape(&v))),
                    }
                    f.write(" ");
                    value.display(f);
                    f.end();
                }
                f.end_block();
            }
            Rewrite(ref rw) => {
                use ast::RewriteFlag::*;
                f.indent();
                f.write("rewrite ");
                f.write(escape(&rw.regex));
                f.write(" ");
                rw.replacement.display(f);
                f.write(match rw.flag {
                    Some(Last) => " last",
                    Some(Break) => " break",
                    Some(Redirect) => " redirect",
                    Some(Permanent) => " permanent",
                    None => "",
                });
                f.end();
            }
            | Root(ref val)
            | Alias(ref val)
            | DefaultType(ref val)
            | ClientMaxBodySize(ref val)
            | Include(ref val)
            | RewriteByLuaFile(ref val)
            | BalancerByLuaFile(ref val)
            | AccessByLuaFile(ref val)
            | HeaderFilterByLuaFile(ref val)
            | ContentByLuaFile(ref val)
            | BodyFilterByLuaFile(ref val)
            | LogByLuaFile(ref val)
            | LuaNeedRequestBody(ref val)
            | SslCertificateByLuaFile(ref val)
            | SslSessionFetchByLuaFile(ref val)
            | SslSessionStoreByLuaFile(ref val)
            | SslCertificate(ref val)
            | SslCertificateKey(ref val)
            | ProxyPass(ref val)
            | ProxyCache(ref val)
            | ProxyCacheKey(ref val)
            | ProxyMethod(ref val)
            | ProxyReadTimeout(ref val)
            | ProxyConnectTimeout(ref val)
            | ProxyHideHeader(ref val)
            | ProxyPassHeader(ref val)
            | ProxyNextUpstreamTries(ref val)
            | ProxyNextUpstreamTimeout(ref val)
            | ServerTokens(ref val)
            | RealIpHeader(ref val)
            => {
                one_arg_dir(self.directive_name(), val, f);
            }
            | EmptyGif
            | Internal
            => {
                f.indent();
                f.write(self.directive_name());
                f.end();
            }
            ErrorPage(ref ep) => {
                use ast::ErrorPageResponse::*;
                f.indent();
                f.write("error_page");
                for code in &ep.codes {
                    f.write(" ");
                    f.fmt(code);
                }
                match ep.response_code {
                    Target => {},
                    Replace(ref code) =>  { f.write(" ="); f.fmt(code); }
                    Redirect(ref code) => { f.write(" ="); f.fmt(code); }
                    Keep => { f.write(" ="); }
                }
                f.write(" ");
                ep.uri.display(f);
                f.end()
            }
            Return(ref ret) => {
                use ast::Return::*;
                f.indent();
                f.write("return ");
                match ret {
                    Redirect { code: None, url } => url.display(f),
                    Redirect { code: Some(code), url } => {
                        f.fmt(&code);
                        f.write(" ");
                        url.display(f);
                    }
                    Text { code, text } => {
                        f.fmt(&code);
                        match text {
                            Some(v) => {
                                f.write(" ");
                                v.display(f);
                            }
                            None => {}
                        }
                    }
                }
                f.end()
            }
            TryFiles(ref tf) => {
                use ast::TryFilesLastOption::*;
                f.indent();
                f.write("try_files ");
                for item in &tf.options {
                    item.display(f);
                    f.write(" ");
                }
                match tf.last_option {
                    Uri(ref v) => v.display(f),
                    NamedLocation(ref loc) => {
                        f.write("@");
                        f.write(&loc);
                    }
                    Code(code) => {
                        f.write("=");
                        f.fmt(&code);
                    }
                }
                f.end();
            }
            Expires(ast::Expires { modified, ref value }) => {
                f.indent();
                f.write("expires ");
                if modified {
                    f.write("modified ");
                }
                value.display(f);
                f.end();
            }
            If(ast::If { ref condition, ref directives, position: _ }) => {
                use ast::IfCondition::*;
                f.indent();
                f.write("if (");
                match condition {
                    NonEmpty(ref v) => v.display(f),
                    Eq(ref v, ref s) => {
                        v.display(f);
                        f.write(" = ");
                        f.write(&escape(s));
                    }
                    Neq(ref v, ref s) => {
                        v.display(f);
                        f.write(" != ");
                        f.write(&escape(s));
                    }
                    RegEq(ref v, ref r, case) => {
                        v.display(f);
                        if *case {
                            f.write(" ~ ");
                        } else {
                            f.write(" ~* ");
                        }
                        f.write(&escape(r));
                    }
                    RegNeq(ref v, ref r, case) => {
                        v.display(f);
                        if *case {
                            f.write(" !~ ");
                        } else {
                            f.write(" !~* ");
                        }
                        f.write(&escape(r));
                    }
                    Exists(ref v) => {
                        f.write("-e ");
                        v.display(f);
                    }
                    NotExists(ref v) => {
                        f.write("!-e ");
                        v.display(f);
                    },
                    FileExists(ref v) => {
                        f.write("-f ");
                        v.display(f);
                    },
                    FileNotExists(ref v) => {
                        f.write("!-f ");
                        v.display(f);
                    },
                    DirExists(ref v) => {
                        f.write("-d ");
                        v.display(f);
                    },
                    DirNotExists(ref v) => {
                        f.write("!-d ");
                        v.display(f);
                    },
                    Executable(ref v) => {
                        f.write("-x ");
                        v.display(f);
                    },
                    NotExecutable(ref v) => {
                        f.write("!-x ");
                        v.display(f);
                    },
                }
                f.write(") ");
                f.start_block();
                for dir in directives {
                    dir.display(f);
                }
                f.end_block();
            }
            Allow(ref source) | Deny(ref source) => {
                use ast::Source::*;
                f.indent();
                f.write(self.directive_name());
                f.write(" ");
                match source {
                    All => f.write("all"),
                    Unix => f.write("unix:"),
                    Ip(ip) => f.fmt(ip),
                    Network(ip, bits) => {
                        f.fmt(ip);
                        f.write("/");
                        f.fmt(bits);
                    }
                }
                f.end();
            }
            ProxyHttpVersion(ver) => {
                use ast::ProxyHttpVersion::*;
                f.indent();
                match ver {
                    V1_0 => f.write("proxy_http_version 1.0"),
                    V1_1 => f.write("proxy_http_version 1.1"),
                }
                f.end();
            }
            ProxyIgnoreHeaders(ref headers) => {
                f.indent();
                f.write(self.directive_name());
                for h in headers {
                    f.write(" ");
                    f.write(&escape(h));
                }
                f.end();
            }
            ProxyCacheValid(ref val) => {
                use ast::ProxyCacheValid::*;
                f.indent();
                f.write(self.directive_name());
                match val {
                    Normal(ref val) => {
                        f.write(" ");
                        val.display(f);
                    }
                    Specific(ref codes, ref val) => {
                        for code in codes {
                            f.write(" ");
                            f.fmt(&code);
                        }
                        f.write(" ");
                        val.display(f);
                    }
                    Any(ref val) => {
                        f.write(" any ");
                        val.display(f);
                    }
                }
                f.end();
            }
            KeepaliveTimeout(ref timeo, ref header_timeo) => {
                f.indent();
                f.write(self.directive_name());
                f.write(" ");
                timeo.display(f);
                if let Some(header_timeo) = header_timeo {
                    f.write(" ");
                    header_timeo.display(f);
                }
                f.end();
            }
            ProxyNextUpstream(ref items) => {
                use ast::ProxyNextUpstreamFlag::*;
                f.indent();
                f.write(self.directive_name());
                for item in items {
                    f.write(" ");
                    f.write(match item {
                        Error => "error",
                        Timeout => "timeout",
                        InvalidHeader => "invalid_header",
                        Http500 => "http_500",
                        Http502 => "http_502",
                        Http503 => "http_503",
                        Http504 => "http_504",
                        Http403 => "http_403",
                        Http404 => "http_404",
                        Http429 => "http_429",
                        NonIdempotent => "non_idempotent",
                        Off => "off",
                    });
                }
                f.end();
            }
            AccessLog(ast::AccessLog::Off) =>  {
                f.indent();
                f.write("access_log off");
                f.end();
            }
            AccessLog(ast::AccessLog::On(ref lg)) =>  {
                f.indent();
                f.write("access_log ");
                lg.path.display(f);
                if let Some(ref fmt) = lg.format {
                    f.write(" ");
                    f.fmt(&escape(fmt));
                }
                if let Some(ref buf) = lg.buffer {
                    f.write(" buffer=");
                    f.fmt(&escape(buf));
                }
                if let Some(ref gzip) = lg.gzip {
                    if let Some(level) = gzip {
                        f.write(" gzip=");
                        f.fmt(&level);
                    } else {
                        f.write(" gzip");
                    }
                }
                if let Some(ref flush) = lg.flush {
                    f.write(" flush=");
                    f.fmt(&escape(flush));
                }
                if let Some(ref condition) = lg.condition {
                    f.write(" if=");
                    condition.display(f);
                }
                f.end();
            }
            SetRealIpFrom(ref source) => {
                use ast::RealIpFrom::*;
                f.indent();
                f.write(self.directive_name());
                f.write(" ");
                match source {
                    Unix => f.write("unix:"),
                    Ip(ip) => f.fmt(ip),
                    Network(ip, bits) => {
                        f.fmt(ip);
                        f.write("/");
                        f.fmt(bits);
                    }
                }
                f.end();
            }
            ErrorLog { ref file, level } => {
                f.indent();
                f.write(self.directive_name());
                f.write(" ");
                file.display(f);
                if let Some(level) = level {
                    use ast::ErrorLevel::*;
                    f.write(" ");
                    f.write(match level {
                        Debug => "debug",
                        Info => "info",
                        Notice => "notice",
                        Warn => "warn",
                        Error => "error",
                        Crit => "crit",
                        Alert => "alert",
                        Emerg => "emerg",
                    });
                }
                f.end();
            }
            Index(ref items) => {
                f.indent();
                f.write("index");
                for item in items {
                    f.write(" ");
                    item.display(f);
                }
                f.end();
            }
        }
    }
}

impl Displayable for ast::Listen {
    fn display(&self, f: &mut Formatter) {
        f.write("listen ");
        self.address.display(f);
        if self.default_server { f.write(" default_server") }
        if self.ssl { f.write(" ssl") }
        match self.ext {
            Some(ast::HttpExt::Http2) => f.write(" http2"),
            Some(ast::HttpExt::Spdy) => f.write(" spdy"),
            None => {}
        }
        if self.proxy_protocol { f.write(" proxy_protocol") }
        if let Some(setfib) = self.setfib {
            f.fmt(&format_args!(" setfib={}", setfib));
        }
        if let Some(fastopen) = self.fastopen {
            f.fmt(&format_args!(" fastopen={}", fastopen));
        }
        if let Some(backlog) = self.backlog {
            f.fmt(&format_args!(" backlog={}", backlog));
        }
        if let Some(rcvbuf) = self.rcvbuf {
            f.fmt(&format_args!(" rcvbuf={}", rcvbuf));
        }
        if let Some(sndbuf) = self.sndbuf {
            f.fmt(&format_args!(" sndbuf={}", sndbuf));
        }
        if self.deferred { f.write(" deferred") }
        if self.bind { f.write(" bind") }
        if let Some(ipv6only) = self.ipv6only {
            f.fmt(&format_args!(" ipv6only={}",
                               if ipv6only { "on" } else { "off" }));
        }
        if self.reuseport { f.write(" reuseport") }
        f.end();
    }
}

impl Displayable for ast::Address {
    fn display(&self, f: &mut Formatter) {
        use ast::Address::*;
        match *self {
            Ip(sa) => f.fmt(&sa),
            StarPort(p) => f.fmt(&format_args!("*:{}", p)),
            Port(p) => f.fmt(&p),
            // TODO(tailhook) escape path
            Unix(ref path) => f.fmt(&format_args!("unix:{}", path.display())),
        }
    }
}

fn to_string<T: Displayable>(v: &T) -> String {
    let style = Style::default();
    let mut formatter = Formatter::new(&style);
    v.display(&mut formatter);
    formatter.into_string()
}

macro_rules! impl_display {
    ($( $typ: ty, )+) => {
        $(
            impl fmt::Display for $typ {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    f.write_str(&to_string(self))
                }
            }
        )+
    };
}

impl_display!(
    ast::Main,
    ast::Listen,
    ast::Address,
    ast::Directive,
    ast::Item,
    value::Value,
);

fn escape(s: &str) -> &str {
    // TODO(tailhook) escape raw value
    return s
}

impl fmt::Display for ast::LocationPattern {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ast::LocationPattern::*;
        match *self {
            Prefix(ref p) => f.write_str(escape(p)),
            Exact(ref p) => write!(f, "= {}", escape(p)),
            FinalPrefix(ref p) => write!(f, "^~ {}", escape(p)),
            Regex(ref p) => write!(f, "~ {}", escape(p)),
            RegexInsensitive(ref p) => write!(f, "~* {}", escape(p)),
            Named(ref name) => {
                write!(f, "{}", escape(&(String::from("@") + name)))
            }
        }
    }
}

impl ast::GzipStatic {
    fn as_str(&self) -> &str {
        use ast::GzipStatic::*;
        match *self {
            On => "on",
            Off => "off",
            Always => "always",
        }
    }
}

impl ast::GzipProxied {
    fn as_str(&self) -> &str {
        use ast::GzipProxied::*;
        match *self {
            Off => "off",
            Expired => "expired",
            NoCache => "no-cache",
            NoStore => "no-store",
            Private => "private",
            NoLastModified => "no_last_modified",
            NoEtag => "no_etag",
            Auth => "auth",
            Any => "any",
        }
    }
}


impl fmt::Display for ast::GzipStatic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl fmt::Display for ast::GzipProxied {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_str().fmt(f)
    }
}
