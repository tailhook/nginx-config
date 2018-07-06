//! Abstract Syntax Tree types

#![allow(missing_docs)] // structures are meant to be self-descriptive
use std::path::PathBuf;
use std::net::{SocketAddr, IpAddr};

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
pub enum IfCondition {
    NonEmpty(Value),
    Eq(Value, String),
    Neq(Value, String),
    RegEq(Value, String, bool),
    RegNeq(Value, String, bool),
    Exists(Value),
    NotExists(Value),
    FileExists(Value),
    FileNotExists(Value),
    DirExists(Value),
    DirNotExists(Value),
    Executable(Value),
    NotExecutable(Value),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct If {
    pub position: (Pos, Pos),
    pub condition: IfCondition,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MapPattern {
    Exact(String),
    Suffix(String),
    StarSuffix(String),
    StarPrefix(String),
    Regex(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Map {
    pub variable: String,
    pub expression: Value,
    pub default: Option<Value>,
    pub hostnames: bool,
    pub volatile: bool,
    pub includes: Vec<String>,
    pub patterns: Vec<(MapPattern, Value)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorPageResponse {
    /// The response code of a target uri
    Target,
    /// Replace with a specified value
    Replace(u32),
    /// Replace with a specified redirect (`=` with codes 301,302,303,307,308)
    Redirect(u32),
    /// Keep original (bare `=`)
    Keep,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorPage {
    pub codes: Vec<u32>,
    pub response_code: ErrorPageResponse,
    pub uri: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Return {
    Redirect { code: Option<u32>, url: Value },
    Text { code: u32, text: Option<Value> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RewriteFlag {
    Last,
    Break,
    Redirect,
    Permanent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rewrite {
    pub regex: String,
    pub replacement: Value,
    pub flag: Option<RewriteFlag>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TryFilesLastOption {
    Uri(Value),
    NamedLocation(String),
    Code(u32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TryFiles {
    pub options: Vec<Value>,
    pub last_option: TryFilesLastOption,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expires {
    pub modified: bool,
    pub value: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    All,
    Unix,
    Ip(IpAddr),
    Network(IpAddr, u8),
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum ProxyHttpVersion {
    V1_0,
    V1_1,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProxyCacheValid {
    Normal(Value),
    Specific(Vec<u32>, Value),
    Any(Value),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProxyNextUpstreamFlag {
    Error,
    Timeout,
    InvalidHeader,
    Http500,
    Http502,
    Http503,
    Http504,
    Http403,
    Http404,
    Http429,
    NonIdempotent,
    Off,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccessLog {
    Off,
    On(AccessLogOptions),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessLogOptions {
    pub path: Value,
    pub format: Option<String>,
    pub buffer: Option<String>,
    pub gzip: Option<Option<u8>>,
    pub flush: Option<String>,
    pub condition: Option<Value>,
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
    ProxyMethod(Value),
    ProxyReadTimeout(Value),
    ProxyConnectTimeout(Value),
    ProxyHideHeader(Value),
    ProxyPassHeader(Value),
    ProxyPassRequestBody(bool),
    ProxyPassRequestHeaders(bool),
    ProxyHttpVersion(ProxyHttpVersion),
    ProxyIgnoreHeaders(Vec<String>),
    ProxyInterceptErrors(bool),
    ProxyBuffering(bool),
    ProxyCache(Value),
    ProxyCacheKey(Value),
    ProxyCacheValid(ProxyCacheValid),
    ProxyNextUpstreamTries(Value),
    ProxyNextUpstreamTimeout(Value),
    ProxyNextUpstream(Vec<ProxyNextUpstreamFlag>),
    Gzip(bool),
    GzipStatic(GzipStatic),
    GzipProxied(Vec<GzipProxied>),
    AddHeader(AddHeader),
    Expires(Expires),
    Root(Value),
    Alias(Value),
    ErrorPage(ErrorPage),
    DefaultType(Value),
    Rewrite(Rewrite),
    Return(Return),
    If(If),
    TryFiles(TryFiles),
    ServerName(Vec<ServerName>),
    Set { variable: String, value: Value },
    Map(Map),
    ClientMaxBodySize(Value),
    Include(Value),
    EmptyGif,
    Internal,
    Etag(bool),
    RecursiveErrorPages(bool),
    ChunkedTransferEncoding(bool),
    KeepaliveTimeout(Value, Option<Value>),
    ServerTokens(Value),
    SslCertificate(Value),
    SslCertificateKey(Value),
    // openresty
    RewriteByLuaFile(Value),
    BalancerByLuaFile(Value),
    AccessByLuaFile(Value),
    HeaderFilterByLuaFile(Value),
    ContentByLuaFile(Value),
    BodyFilterByLuaFile(Value),
    LogByLuaFile(Value),
    LuaNeedRequestBody(Value),
    SslCertificateByLuaFile(Value),
    SslSessionFetchByLuaFile(Value),
    SslSessionStoreByLuaFile(Value),
    // access module
    Allow(Source),
    Deny(Source),
    // log module
    AccessLog(AccessLog),
}

impl Item {

    pub fn directive_name(&self) -> &'static str {
        use self::Item::*;
        match *self {
            Daemon(..) => "daemon",
            MasterProcess(..) => "master_process",
            WorkerProcesses(..) => "worker_processes",
            Http(..) => "http",
            Server(..) => "server",
            Location(..) => "location",
            Listen(..) => "listen",
            ProxyPass(..) => "proxy_pass",
            ProxySetHeader {..} => "proxy_set_header",
            ProxyMethod {..} => "proxy_method",
            ProxyReadTimeout {..} => "proxy_read_timeout",
            ProxyConnectTimeout {..} => "proxy_connect_timeout",
            ProxyHideHeader {..} => "proxy_hide_header",
            ProxyPassHeader {..} => "proxy_pass_header",
            ProxyPassRequestHeaders(..) => "proxy_pass_request_headers",
            ProxyPassRequestBody(..) => "proxy_pass_request_body",
            ProxyHttpVersion {..} => "proxy_http_version",
            ProxyIgnoreHeaders(..) => "proxy_ignore_headers",
            ProxyInterceptErrors(..) => "proxy_intercept_errors",
            ProxyBuffering(..) => "proxy_buffering",
            ProxyCache(..) => "proxy_cache",
            ProxyCacheKey(..) => "proxy_cache_key",
            ProxyCacheValid(..) => "proxy_cache_valid",
            ProxyNextUpstreamTries(..) => "proxy_next_upstream_tries",
            ProxyNextUpstreamTimeout(..) => "proxy_next_upstream_timeout",
            ProxyNextUpstream(..) => "proxy_next_upstream",
            Gzip(..) => "gzip",
            GzipStatic(..) => "gzip_static",
            GzipProxied(..) => "gzip_proxied",
            AddHeader(..) => "add_header",
            Expires(..) => "expires",
            Root(..) => "root",
            Alias(..) => "alias",
            ErrorPage(..) => "error_page",
            DefaultType(..) => "default_type",
            Rewrite(..) => "rewrite",
            Return(..) => "return",
            If(..) => "if",
            TryFiles(..) => "try_files",
            ServerName(..) => "server_name",
            Set { .. } => "set",
            Map(..) => "map",
            ClientMaxBodySize(..) => "client_max_body_size",
            Include(..) => "include",
            EmptyGif => "empty_gif",
            Internal => "internal",
            Etag(..) => "etag",
            RecursiveErrorPages(..) => "recursive_error_pages",
            ChunkedTransferEncoding(..) => "chunked_transfer_encoding",
            KeepaliveTimeout(..) => "keepalive_timeout",
            ServerTokens(..) => "server_tokens",
            SslCertificate(..) => "ssl_certificate",
            SslCertificateKey(..) => "ssl_certificate_key",
            // openresty
            RewriteByLuaFile(..) => "rewrite_by_lua_file",
            BalancerByLuaFile(..) => "balancer_by_lua_file",
            AccessByLuaFile(..) => "access_by_lua_file",
            HeaderFilterByLuaFile(..) => "header_filter_by_lua_file",
            ContentByLuaFile(..) => "content_by_lua_file",
            BodyFilterByLuaFile(..) => "body_filter_by_lua_file",
            LogByLuaFile(..) => "log_by_lua_file",
            LuaNeedRequestBody(..) => "lua_need_request_body",
            SslCertificateByLuaFile(..) => "ssl_certificate_by_lua_file",
            SslSessionFetchByLuaFile(..) => "ssl_session_fetch_by_lua_file",
            SslSessionStoreByLuaFile(..) => "ssl_session_store_by_lua_file",
            // access module
            Allow(..) => "allow",
            Deny(..) => "deny",
            // log module
            AccessLog(..) => "access_log",
        }
    }

    pub fn children(&self) -> Option<&[Directive]> {
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
            ProxyPassRequestHeaders(..) => None,
            ProxyPassRequestBody(..) => None,
            ProxySetHeader {..} => None,
            ProxyMethod {..} => None,
            ProxyReadTimeout {..} => None,
            ProxyConnectTimeout {..} => None,
            ProxyHideHeader {..} => None,
            ProxyPassHeader {..} => None,
            ProxyHttpVersion {..} => None,
            ProxyIgnoreHeaders(..) => None,
            ProxyInterceptErrors(..) => None,
            ProxyBuffering(..) => None,
            ProxyCache(..) => None,
            ProxyCacheKey(..) => None,
            ProxyCacheValid(..) => None,
            ProxyNextUpstreamTries(..) => None,
            ProxyNextUpstreamTimeout(..) => None,
            ProxyNextUpstream(..) => None,
            Gzip(..) => None,
            GzipStatic(..) => None,
            GzipProxied(..) => None,
            AddHeader(..) => None,
            Expires(..) => None,
            Root(..) => None,
            Alias(..) => None,
            ErrorPage(..) => None,
            DefaultType(..) => None,
            Rewrite(..) => None,
            Return(..) => None,
            If(ref val) => Some(&val.directives),
            TryFiles(..) => None,
            ServerName(..) => None,
            Set { .. } => None,
            Map(..) => None,
            ClientMaxBodySize(..) => None,
            Include(..) => None,
            EmptyGif => None,
            Internal => None,
            Etag(..) => None,
            RecursiveErrorPages(..) => None,
            ChunkedTransferEncoding(..) => None,
            KeepaliveTimeout(..) => None,
            ServerTokens(..) => None,
            SslCertificate(..) => None,
            SslCertificateKey(..) => None,
            // openresty
            RewriteByLuaFile(..) => None,
            BalancerByLuaFile(..) => None,
            AccessByLuaFile(..) => None,
            HeaderFilterByLuaFile(..) => None,
            ContentByLuaFile(..) => None,
            BodyFilterByLuaFile(..) => None,
            LogByLuaFile(..) => None,
            LuaNeedRequestBody(..) => None,
            SslCertificateByLuaFile(..) => None,
            SslSessionFetchByLuaFile(..) => None,
            SslSessionStoreByLuaFile(..) => None,
            // access
            Allow(..) => None,
            Deny(..) => None,
            // log module
            AccessLog(..) => None,
        }
    }

    pub fn children_mut(&mut self) -> Option<&mut Vec<Directive>> {
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
            ProxyMethod {..} => None,
            ProxyReadTimeout {..} => None,
            ProxyConnectTimeout {..} => None,
            ProxyHideHeader {..} => None,
            ProxyPassHeader {..} => None,
            ProxyPassRequestHeaders(_) => None,
            ProxyPassRequestBody(_) => None,
            ProxyHttpVersion {..} => None,
            ProxyIgnoreHeaders {..} => None,
            ProxyInterceptErrors {..} => None,
            ProxyBuffering(..) => None,
            ProxyCache {..} => None,
            ProxyCacheKey {..} => None,
            ProxyCacheValid {..} => None,
            ProxyNextUpstreamTries(..) => None,
            ProxyNextUpstreamTimeout(..) => None,
            ProxyNextUpstream(..) => None,
            Gzip(..) => None,
            GzipStatic(..) => None,
            GzipProxied(..) => None,
            AddHeader(..) => None,
            Expires(..) => None,
            Root(..) => None,
            Alias(..) => None,
            ErrorPage(..) => None,
            DefaultType(..) => None,
            Rewrite(..) => None,
            Return(..) => None,
            If(ref mut val) => Some(&mut val.directives),
            TryFiles(..) => None,
            ServerName(..) => None,
            Set { .. } => None,
            Map(..) => None,
            ClientMaxBodySize(..) => None,
            Include(..) => None,
            EmptyGif => None,
            Internal => None,
            Etag(..) => None,
            RecursiveErrorPages(..) => None,
            ChunkedTransferEncoding(..) => None,
            KeepaliveTimeout(..) => None,
            ServerTokens(..) => None,
            SslCertificate(..) => None,
            SslCertificateKey(..) => None,
            // openresty
            RewriteByLuaFile(..) => None,
            BalancerByLuaFile(..) => None,
            AccessByLuaFile(..) => None,
            HeaderFilterByLuaFile(..) => None,
            ContentByLuaFile(..) => None,
            BodyFilterByLuaFile(..) => None,
            LogByLuaFile(..) => None,
            LuaNeedRequestBody(..) => None,
            SslCertificateByLuaFile(..) => None,
            SslSessionFetchByLuaFile(..) => None,
            SslSessionStoreByLuaFile(..) => None,
            // access
            Allow(..) => None,
            Deny(..) => None,
            // log module
            AccessLog(..) => None,
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
            ProxyMethod(ref mut v) => f(v),
            ProxyReadTimeout(ref mut v) => f(v),
            ProxyConnectTimeout(ref mut v) => f(v),
            ProxyHideHeader(ref mut v) => f(v),
            ProxyPassHeader(ref mut v) => f(v),
            ProxyCache(ref mut v) => f(v),
            ProxyCacheKey(ref mut v) => f(v),
            ProxyCacheValid(self::ProxyCacheValid::Normal(ref mut v)) => f(v),
            ProxyCacheValid(self::ProxyCacheValid::Specific(_, ref mut v)) => f(v),
            ProxyCacheValid(self::ProxyCacheValid::Any(ref mut v)) => f(v),
            ProxyPassRequestHeaders(_) => {},
            ProxyPassRequestBody(_) => {},
            ProxyHttpVersion(..) => {},
            ProxyIgnoreHeaders(..) => {},
            ProxyInterceptErrors(..) => {},
            ProxyBuffering(..) => {},
            ProxyNextUpstreamTries(ref mut v) => f(v),
            ProxyNextUpstreamTimeout(ref mut v) => f(v),
            ProxyNextUpstream(_) => {},
            Gzip(_) => {},
            GzipStatic(_) => {},
            GzipProxied(_) => {},
            AddHeader(self::AddHeader { ref mut field, ref mut value, .. })
            => {
                f(field);
                f(value);
            }
            Expires(self::Expires { ref mut value, .. }) => f(value),
            Root(ref mut v) => f(v),
            Alias(ref mut v) => f(v),
            ErrorPage(::ast::ErrorPage { ref mut uri, .. }) => f(uri),
            DefaultType(ref mut v) => f(v),
            Rewrite(ref mut rw) => f(&mut rw.replacement),
            Return(::ast::Return::Redirect { ref mut url, .. }) => f(url),
            Return(::ast::Return::Text { text: Some(ref mut t), .. }) => f(t),
            Return(::ast::Return::Text { text: None, .. }) => {},
            If(self::If { ref mut condition, .. }) => {
                use self::IfCondition::*;
                match condition {
                    NonEmpty(ref mut v) => f(v),
                    Eq(ref mut v, _) => f(v),
                    Neq(ref mut v, _) => f(v),
                    RegEq(ref mut v, _, _) => f(v),
                    RegNeq(ref mut v, _, _) => f(v),
                    Exists(ref mut v) => f(v),
                    NotExists(ref mut v) => f(v),
                    FileExists(ref mut v) => f(v),
                    FileNotExists(ref mut v) => f(v),
                    DirExists(ref mut v) => f(v),
                    DirNotExists(ref mut v) => f(v),
                    Executable(ref mut v) => f(v),
                    NotExecutable(ref mut v) => f(v),
                }
            },
            TryFiles(ref mut tf) => {
                for opt in &mut tf.options {
                    f(opt);
                }
                match tf.last_option {
                    TryFilesLastOption::Uri(ref mut v) => f(v),
                    TryFilesLastOption::NamedLocation(..) => {},
                    TryFilesLastOption::Code(..) => {},
                }
            },
            Include(ref mut v) => f(v),
            EmptyGif => {}
            Internal => {}
            Etag(..) => {}
            RecursiveErrorPages(..) => {}
            ChunkedTransferEncoding(..) => {}
            KeepaliveTimeout(ref mut a, ref mut b) => {
                f(a);
                if let Some(b) = b {
                    f(b);
                }
            }
            ServerTokens(ref mut v) => f(v),
            SslCertificate(ref mut v) => f(v),
            SslCertificateKey(ref mut v) => f(v),
            ServerName(_) => {},
            Set { ref mut value, .. } => f(value),
            Map(::ast::Map {
                ref mut expression,
                ref mut default,
                ref mut patterns,
                ..
            }) => {
                f(expression);
                if let Some(ref mut def) = default {
                    f(def);
                }
                for (_, v) in patterns {
                    f(v);
                }
            }
            ClientMaxBodySize(ref mut v) => f(v),
            // openresty
            RewriteByLuaFile(ref mut v) => f(v),
            BalancerByLuaFile(ref mut v) => f(v),
            AccessByLuaFile(ref mut v) => f(v),
            HeaderFilterByLuaFile(ref mut v) => f(v),
            ContentByLuaFile(ref mut v) => f(v),
            BodyFilterByLuaFile(ref mut v) => f(v),
            LogByLuaFile(ref mut v) => f(v),
            LuaNeedRequestBody(ref mut v) => f(v),
            SslCertificateByLuaFile(ref mut v) => f(v),
            SslSessionFetchByLuaFile(ref mut v) => f(v),
            SslSessionStoreByLuaFile(ref mut v) => f(v),
            // access
            Allow(..) => {},
            Deny(..) => {},
            // log module
            AccessLog(::ast::AccessLog::Off) => {},
            AccessLog(::ast::AccessLog::On(ref mut lg)) => {
                f(&mut lg.path);
                lg.condition.as_mut().map(f);
            },
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
