#![allow(missing_docs)] // structures are meant to be self-descriptive
use std::path::PathBuf;
use std::net::SocketAddr;

use position::Pos;
use value::Value;


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
}
