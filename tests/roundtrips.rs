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
// not working yet
//#[test] fn few_locations() { roundtrip("few_locations"); }
