//! Nginx Config Parser (unofficial
//! ===============================
//!
//! This library contains parser and formatter of nginx config format
//! as well as AST types.
//!
//! [Docs](https://docs.rs/nginx-config/) |
//! [Github](https://github.com/tailhook/nginx-config/) |
//! [Crate](https://crates.io/crates/nginx-config)
//!
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]

extern crate combine;
#[macro_use] extern crate failure;
#[cfg(test)] #[macro_use] extern crate pretty_assertions;

mod ast;
mod error;
mod grammar;
mod position;
mod tokenizer;
