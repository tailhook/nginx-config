use ast;
use std::fmt;
use format::{Displayable, Formatter, Style};

use value;

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
            Daemon(opt) => {
                f.indent();
                f.write("daemon ");
                f.write(if opt { "on" } else { "off" });
                f.end();
            }
            MasterProcess(opt) => {
                f.indent();
                f.write("master_process ");
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
            Listen(ref lst) => {
                f.indent();
                lst.display(f);
            }
            ProxyPass(ref val) => {
                f.indent();
                f.write("proxy_pass ");
                val.display(f);
                f.end();
            }
            ProxySetHeader { ref field, ref value } => {
                f.indent();
                f.write("proxy_set_header ");
                field.display(f);
                f.write(" ");
                value.display(f);
                f.end();
            }
            Gzip(opt) => {
                f.indent();
                f.write("gzip ");
                f.write(if opt { "on" } else { "off" });
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
            | Root(ref val)
            | Alias(ref val)
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
            => {
                one_arg_dir(self.directive_name(), val, f);
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
