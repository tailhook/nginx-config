extern crate nginx_config;
#[cfg(test)] #[macro_use] extern crate pretty_assertions;

use nginx_config::parse_main;

fn roundtrip(value: &str) {
    let ast = parse_main(&value).unwrap();
    assert_eq!(ast.to_string(), value);
}

#[test] fn simple() { roundtrip("set $xy something;\n"); }
#[test] fn from_vars() { roundtrip("set $real_id id-$request_id;\n"); }
#[test] fn from_regex() { roundtrip("set $real_id $1;\n"); }
