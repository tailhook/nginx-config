use std::net::IpAddr;

use combine::{choice, Parser};
use combine::error::StreamError;
use combine::easy::Error;

use crate::ast::{Item, RealIpFrom};
use crate::grammar::{value, bool};
use crate::helpers::{semi, ident, string};
use crate::tokenizer::{TokenStream, Token};


fn parse_source<'a>(val: Token<'a>)
    -> Result<RealIpFrom, Error<Token<'a>, Token<'a>>>
{
    let value = val.value;
    if value == "unix:" {
        return Ok(RealIpFrom::Unix);
    }
    let mut pair = value.splitn(2, '/');
    let addr = pair.next().unwrap().parse::<IpAddr>()?;
    if let Some(net) = pair.next() {
        let subnet = net.parse::<u8>()
            .map_err(|e| Error::unexpected_message(
                format!("invalid subnet: {}", e)))?;
        return Ok(RealIpFrom::Network(addr, subnet));
    } else {
        return Ok(RealIpFrom::Ip(addr));
    }
}

pub fn directives<'a>()
    -> impl Parser<Output=Item, Input=TokenStream<'a>>
{
    choice((
        ident("real_ip_header").with(value())
            .skip(semi()).map(Item::RealIpHeader),
        ident("real_ip_recursive").with(bool())
            .skip(semi()).map(Item::RealIpRecursive),
        ident("set_real_ip_from")
            .with(string().and_then(parse_source))
            .skip(semi()).map(Item::SetRealIpFrom),
    ))
}
