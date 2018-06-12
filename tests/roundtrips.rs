extern crate nginx_config;
#[cfg(test)] #[macro_use] extern crate pretty_assertions;

use std::io::Read;
use std::fs::File;

use nginx_config::parse_main;

fn roundtrip(filename: &str) {
    let mut buf = String::with_capacity(1024);
    let path = format!("tests/configs/{}.conf", filename);
    let mut f = File::open(&path).unwrap();
    f.read_to_string(&mut buf).unwrap();
    let ast = parse_main(&buf).unwrap();
    assert_eq!(ast.to_string(), buf);
}

#[test] fn minimal() { roundtrip("minimal"); }
#[test] fn master_process() { roundtrip("master_process"); }
#[test] fn worker_processes_auto() { roundtrip("worker_processes_auto"); }
#[test] fn worker_processes_7() { roundtrip("worker_processes_7"); }
#[test] fn worker_processes_13() { roundtrip("worker_processes_13"); }
#[test] fn http() { roundtrip("http"); }
#[test] fn listen() { roundtrip("listen"); }
#[test] fn proxy() { roundtrip("proxy"); }
#[test] fn location() { roundtrip("location"); }
#[test] fn two_locations() { roundtrip("two_locations"); }
#[test] fn gzip() { roundtrip("gzip"); }
#[test] fn gzip_proxied() { roundtrip("gzip_proxied"); }
#[test] fn add_header() { roundtrip("add_header"); }
#[test] fn root() { roundtrip("root"); }
#[test] fn alias() { roundtrip("alias"); }
#[test] fn client_max_body_size() { roundtrip("client_max_body_size"); }
#[test] fn openresty() { roundtrip("openresty"); }
#[test] fn include() { roundtrip("include"); }
#[test] fn map() { roundtrip("map"); }
#[test] fn error_pages() { roundtrip("error_pages"); }
#[test] fn returns() { roundtrip("return"); }
#[test] fn ssl() { roundtrip("ssl"); }
#[test] fn rewrite() { roundtrip("rewrite"); }
#[test] fn try_files() { roundtrip("try_files"); }
#[test] fn empty_gif() { roundtrip("empty_gif"); }
#[test] fn internal() { roundtrip("internal"); }
#[test] fn expires() { roundtrip("expires"); }
#[test] fn ifs() { roundtrip("ifs"); }
#[test] fn allow_deny() { roundtrip("allow_deny"); }
#[test] fn etag() { roundtrip("etag"); }
// not working yet
//#[test] fn few_locations() { roundtrip("few_locations"); }
