use std::marker::PhantomData;

use combine::{Parser, ConsumedResult, satisfy, StreamOnce};
use combine::error::{Tracked};
use combine::stream::easy::{Error, Errors, Info};

use tokenizer::{TokenStream, Kind, Token};
use position::Pos;


#[derive(Debug, Clone)]
pub struct TokenMatch<'a> {
    kind: Kind,
    phantom: PhantomData<&'a u8>,
}

#[derive(Debug, Clone)]
pub struct Value<'a> {
    kind: Kind,
    value: &'static str,
    phantom: PhantomData<&'a u8>,
}

#[derive(Debug, Clone)]
pub struct Prefix<'a> {
    value: &'static str,
    phantom: PhantomData<&'a u8>,
}

pub fn kind<'x>(kind: Kind) -> TokenMatch<'x> {
    TokenMatch {
        kind: kind,
        phantom: PhantomData,
    }
}

pub fn semi<'x>() -> TokenMatch<'x> {
    kind(Kind::Semicolon)
}

pub fn ident<'x>(value: &'static str) -> Value<'x> {
    Value {
        kind: Kind::String,
        value: value,
        phantom: PhantomData,
    }
}

pub fn prefix<'x>(prefix: &'static str) -> Prefix<'x> {
    Prefix {
        value: prefix,
        phantom: PhantomData,
    }
}

pub fn string<'x>() -> TokenMatch<'x> {
    kind(Kind::String)
}

impl<'a> Parser for TokenMatch<'a> {
    type Input = TokenStream<'a>;
    type Output = Token<'a>;
    type PartialState = ();

    #[inline]
    fn parse_lazy(&mut self, input: &mut Self::Input)
        -> ConsumedResult<Self::Output, Self::Input>
    {
        satisfy(|c: Token<'a>| c.kind == self.kind).parse_lazy(input)
    }

    fn add_error(&mut self,
        error: &mut Tracked<Errors<Token<'a>, Token<'a>, Pos>>)
    {
        error.error.add_error(Error::Expected(Info::Owned(
            format!("{:?}", self.kind))));
    }
}

impl<'a> Parser for Value<'a> {
    type Input = TokenStream<'a>;
    type Output = Token<'a>;
    type PartialState = ();

    #[inline]
    fn parse_lazy(&mut self, input: &mut Self::Input)
        -> ConsumedResult<Self::Output, Self::Input>
    {
        satisfy(|c: Token<'a>| {
            c.kind == self.kind && c.value == self.value
        }).parse_lazy(input)
    }

    fn add_error(&mut self,
        error: &mut Tracked<<Self::Input as StreamOnce>::Error>)
    {
        error.error.add_error(Error::Expected(Info::Borrowed(self.value)));
    }
}

impl<'a> Parser for Prefix<'a> {
    type Input = TokenStream<'a>;
    type Output = &'a str;
    type PartialState = ();

    #[inline]
    fn parse_lazy(&mut self, input: &mut Self::Input)
        -> ConsumedResult<Self::Output, Self::Input>
    {
        satisfy(|c: Token<'a>| {
            c.kind == Kind::String && c.value.starts_with(self.value)
        })
        .map(|t: Token<'a>| &t.value[self.value.len()..])
        .parse_lazy(input)
    }

    fn add_error(&mut self,
        error: &mut Tracked<<Self::Input as StreamOnce>::Error>)
    {
        error.error.add_error(Error::Expected(Info::Borrowed(self.value)));
    }
}
