Nginx Config Parser (unofficial)
================================

[Documentation](https://docs.rs/nginx-config) |
[Github](https://github.com/tailhook/nginx-config) |
[Crate](https://crates.io/crates/nginx-config)

A parser, formatter and AST for nginx configs.

The goal is to:

1. Validate nginx config, including custom rules
2. Validate partial configs like location or server directive
3. Extract facts from nginx config, like domains served or upstream list
4. Support simple config rewritings like replace variable with actual value
5. Generate nginx configs

We're starting with small subset of the nginx directives. And we're unlikely
be supporting all of the config in the nearest future. But PR's on additional
directives are accepted.

Non-goals:

* Being replacement for ``nginx -t``


License
=======

Licensed under either of

* Apache License, Version 2.0,
  (./LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license (./LICENSE-MIT or http://opensource.org/licenses/MIT)
  at your option.

Contribution
------------

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.

