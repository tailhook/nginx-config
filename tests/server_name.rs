extern crate nginx_config;
#[cfg(test)] #[macro_use] extern crate pretty_assertions;

use nginx_config::parse_main;

fn roundtrip(value: &str) {
    let ast = parse_main(&value).unwrap();
    assert_eq!(ast.to_string(), value);
}

#[test] fn exact()    { roundtrip("server_name devd.io;\n"); }
#[test] fn suffix()   { roundtrip("server_name .devd.io;\n"); }
#[test] fn multiple() { roundtrip("server_name example.com .devd.io;\n"); }
#[test] fn star_pre() { roundtrip("server_name *.devd.io;\n"); }
#[test] fn star_suf() { roundtrip("server_name mail.*;\n"); }
#[test] fn regex()    { roundtrip("server_name ~^www\\.[.*]\\.devd\\.io;\n"); }
