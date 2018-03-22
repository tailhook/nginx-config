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

fn simple_block(f: &mut Formatter, name: &str, directives: &[ast::Directive]) {
    f.fmt(&format_args!("{} ", name));
    f.start_block();
    for dir in directives {
        dir.display(f);
    }
    f.end_block();
}

impl Displayable for ast::Item {
    fn display(&self, f: &mut Formatter) {
        use ast::Item::*;
        f.indent();
        match *self {
            Daemon(opt) => {
                f.write("daemon ");
                f.write(if opt { "on" } else { "off" });
                f.end();
            }
            MasterProcess(opt) => {
                f.write("master_process ");
                f.write(if opt { "on" } else { "off" });
                f.end();
            }
            WorkerProcesses(ast::WorkerProcesses::Auto) => {
                f.write("worker_processes auto");
                f.end();
            }
            WorkerProcesses(ast::WorkerProcesses::Exact(n)) => {
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
            Listen(ref lst) => {
                lst.display(f);
            }
            ProxyPass(ref val) => {
                f.write("proxy_pass ");
                val.display(f);
                f.end();
            }
            ProxySetHeader { ref field, ref value } => {
                f.write("proxy_set_header ");
                field.display(f);
                f.write(" ");
                value.display(f);
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
    value::Value,
);
