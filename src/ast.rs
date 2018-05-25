//! Abstract Syntax Tree types

#![allow(missing_docs)] // structures are meant to be self-descriptive
use std::path::PathBuf;
use std::net::SocketAddr;

pub use value::{Value};
use position::Pos;
use visitors::{DirectiveIter};


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Main {
    pub directives: Vec<Directive>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Directive {
    pub position: Pos,
    pub item: Item,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkerProcesses {
    Auto,
    Exact(u32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Http {
    pub position: (Pos, Pos),
    pub directives: Vec<Directive>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Server {
    pub position: (Pos, Pos),
    pub directives: Vec<Directive>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Address {
    Ip(SocketAddr),
    StarPort(u16),
    Port(u16),
    Unix(PathBuf),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpExt {
    Http2,
    Spdy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listen {
    pub address: Address,
    pub default_server: bool,
    pub ssl: bool,
    pub ext: Option<HttpExt>,
    pub proxy_protocol: bool,
    pub setfib: Option<i32>,
    pub fastopen: Option<u32>,
    pub backlog: Option<i32>,
    pub rcvbuf: Option<u64>,
    pub sndbuf: Option<u64>,
    // TODO(tailhook) Not sure
    // accept_filter: String,
    pub deferred: bool,
    pub bind: bool,
    pub ipv6only: Option<bool>,
    pub reuseport: bool,
    // TODO(tailhook) requires complex parser
    // so_keepalive: Option<KeepAlive>,
}

impl Listen {
    pub fn new(address: Address) -> Listen {
        Listen {
            address,
            default_server: false,
            ssl: false,
            ext: None,
            proxy_protocol: false,
            setfib: None,
            fastopen: None,
            backlog: None,
            rcvbuf: None,
            sndbuf: None,
            deferred: false,
            bind: false,
            ipv6only: None,
            reuseport: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocationPattern {
    Prefix(String),
    Exact(String),
    FinalPrefix(String),
    Regex(String),
    RegexInsensitive(String),
    Named(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    pub position: (Pos, Pos),
    pub pattern: LocationPattern,
    pub directives: Vec<Directive>,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum GzipStatic {
    On,
    Off,
    Always,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum GzipProxied {
    Off,
    Expired,
    NoCache,
    NoStore,
    Private,
    NoLastModified,
    NoEtag,
    Auth,
    Any,
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddHeader {
    pub field: Value,
    pub value: Value,
    pub always: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerName {
    Exact(String),
    Suffix(String),
    StarSuffix(String),
    StarPrefix(String),
    Regex(String),
}

/// The enum which represents nginx config directive
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    Daemon(bool),
    MasterProcess(bool),
    WorkerProcesses(WorkerProcesses),
    Http(Http),
    Server(Server),
    Location(Location),
    Listen(Listen),
    ProxyPass(Value),
    ProxySetHeader { field: Value, value: Value },
    Gzip(bool),
    GzipStatic(GzipStatic),
    GzipProxied(Vec<GzipProxied>),
    AddHeader(AddHeader),
    Root(Value),
    Alias(Value),
    ServerName(Vec<ServerName>),
    Set { variable: String, value: Value },
    ClientMaxBodySize(Value),
}

impl Item {

    pub(crate) fn children(&self) -> Option<&[Directive]> {
        use self::Item::*;
        match *self {
            Daemon(_) => None,
            MasterProcess(_) => None,
            WorkerProcesses(_) => None,
            Http(ref h) => Some(&h.directives[..]),
            Server(ref s) => Some(&s.directives[..]),
            Location(ref l) => Some(&l.directives[..]),
            Listen(_) => None,
            ProxyPass(_) => None,
            ProxySetHeader {..} => None,
            Gzip(..) => None,
            GzipStatic(..) => None,
            GzipProxied(..) => None,
            AddHeader(..) => None,
            Root(..) => None,
            Alias(..) => None,
            ServerName(..) => None,
            Set { .. } => None,
            ClientMaxBodySize(..) => None,
        }
    }

    pub(crate) fn children_mut(&mut self) -> Option<&mut Vec<Directive>> {
        use self::Item::*;
        match *self {
            Daemon(_) => None,
            MasterProcess(_) => None,
            WorkerProcesses(_) => None,
            Http(ref mut h) => Some(&mut h.directives),
            Server(ref mut s) => Some(&mut s.directives),
            Location(ref mut l) => Some(&mut l.directives),
            Listen(_) => None,
            ProxyPass(_) => None,
            ProxySetHeader {..} => None,
            Gzip(..) => None,
            GzipStatic(..) => None,
            GzipProxied(..) => None,
            AddHeader(..) => None,
            Root(..) => None,
            Alias(..) => None,
            ServerName(..) => None,
            Set { .. } => None,
            ClientMaxBodySize(..) => None,
        }
    }

    /// Executes function on all the Value things (not recursively)
    ///
    /// This is useful for substituting variables.
    ///
    /// The callback isn't called for directives inside the  `{ block }`, so
    /// this function might be better used with [`visit_mutable`]
    ///
    /// [`visit_mutable`]: ../visitors/fn.visit_mutable.html
    pub(crate) fn visit_values_mut<F>(&mut self, mut f: F)
        where F: FnMut(&mut Value)
    {
        use self::Item::*;
        match *self {
            Daemon(_) => {},
            MasterProcess(_) => {},
            WorkerProcesses(_) => {},
            Http(_) => {},
            Server(_) => {},
            Location(_) => {},
            Listen(_) => {},
            ProxyPass(ref mut v) => f(v),
            ProxySetHeader { ref mut field, ref mut value } => {
                f(field);
                f(value);
            }
            Gzip(_) => {},
            GzipStatic(_) => {},
            GzipProxied(_) => {},
            AddHeader(self::AddHeader { ref mut field, ref mut value, .. })
            => {
                f(field);
                f(value);
            }
            Root(ref mut v) => f(v),
            Alias(ref mut v) => f(v),
            ServerName(_) => {},
            Set { ref mut value, .. } => f(value),
            ClientMaxBodySize(ref mut v) => f(v),
        }
    }
}

impl Directive {
    /// Executes function on all the Value things (not recursively)
    ///
    /// This is useful for substituting variables.
    ///
    /// The callback isn't called for directives inside the  `{ block }`, so
    /// this function might be better used with [`visit_mutable`]
    ///
    /// [`visit_mutable`]: ../visitors/fn.visit_mutable.html
    pub fn visit_values_mut<F>(&mut self, f: F)
        where F: FnMut(&mut Value)
    {
        self.item.visit_values_mut(f)
    }
}

impl Main {
    pub fn all_directives(&self) -> DirectiveIter {
        DirectiveIter::depth_first(&self.directives)
    }
}
