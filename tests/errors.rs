extern crate nginx_config;
#[cfg(test)] #[macro_use] extern crate pretty_assertions;

use std::io::Read;
use std::fs::File;

use nginx_config::parse_main;


fn test_error(filename: &str) {
    let mut buf = String::with_capacity(1024);
    let path = format!("tests/errors/{}.txt", filename);
    let mut f = File::open(&path).unwrap();
    f.read_to_string(&mut buf).unwrap();
    let mut iter = buf.splitn(2, "\n---\n");
    let graphql = iter.next().unwrap();
    let expected = iter.next().expect("file should contain error message");
    let err = parse_main(graphql).unwrap_err();
    assert_eq!(err.to_string(), expected);
}

#[test] fn invalid_directive() { test_error("invalid_directive"); }
#[test] fn invalid_directive_with_newline() {
    test_error("invalid_directive_with_newline");
}
