extern crate nginx_config;
#[cfg(test)] #[macro_use] extern crate pretty_assertions;

use nginx_config::parse_main;

fn list(value: &str) -> Vec<String> {
    let ast = parse_main(&value).unwrap();
    ast.directives().map(|s| s.to_string()).collect()
}

#[test]
fn iterlocation() {
    assert_eq!(list(r#"
        server_name devd.io;
        location / {
            root /public;
        }
    "#), vec![
        "server_name devd.io;\n",
        "location / {\n    root /public;\n}\n",
        "root /public;\n",
    ]);
}
