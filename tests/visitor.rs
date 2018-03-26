extern crate nginx_config;
#[cfg(test)] #[macro_use] extern crate pretty_assertions;

use nginx_config::{parse_main, visitors};

fn list(value: &str) -> Vec<String> {
    let ast = parse_main(&value).unwrap();
    ast.all_directives().map(|s| s.to_string()).collect()
}

fn replace_vars<'a, F, S>(text: &str, f: F)
    -> String
    where F: FnMut(&str) -> Option<S>,
          S: AsRef<str> + Into<String> + 'a,
{
    let mut ast = parse_main(&text).unwrap();
    visitors::replace_vars(&mut ast.directives, f);
    ast.to_string()
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

#[test]
fn test_replace_vars() {
    assert_eq!(replace_vars(r#"
        location / {
            root /public/$dir;
        }
    "#, |x| if x == "dir" { Some("some/path") } else { None }), "\
        location / {\n    \
            root /public/some/path;\n\
        }\n\
        ");
}
