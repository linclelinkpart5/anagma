mod producers;

use std::convert::TryFrom;

use crate::metadata::types::MetaVal;
use crate::updated_scripting::Error;

pub use self::producers::*;

pub enum Producer<'a, I>
where
    I: Iterator<Item = Result<MetaVal, Error>>,
{
    Source(Source<'a>),
    Fixed(Fixed),
    Raw(Raw),
    Flatten(Flatten<I>),
}

impl<'a, I> From<Vec<MetaVal>> for Producer<'a, I>
where
    I: Iterator<Item = Result<MetaVal, Error>>,
{
    fn from(v: Vec<MetaVal>) -> Self {
        Self::Fixed(v.into())
    }
}

impl<'a, I> From<Vec<Result<MetaVal, Error>>> for Producer<'a, I>
where
    I: Iterator<Item = Result<MetaVal, Error>>,
{
    fn from(v: Vec<Result<MetaVal, Error>>) -> Self {
        Self::Raw(v.into())
    }
}

impl<'a, I> TryFrom<Producer<'a, I>> for Vec<MetaVal>
where
    I: Iterator<Item = Result<MetaVal, Error>>,
{
    type Error = Error;

    fn try_from(vp: Producer<'a, I>) -> Result<Self, Self::Error> {
        vp.collect::<Result<Vec<_>, _>>()
    }
}

impl<'a, I> From<Producer<'a, I>> for Vec<Result<MetaVal, Error>>
where
    I: Iterator<Item = Result<MetaVal, Error>>,
{
    fn from(vp: Producer<'a, I>) -> Self {
        vp.collect()
    }
}

impl<'a, I> Producer<'a, I>
where
    I: Iterator<Item = Result<MetaVal, Error>>,
{
    pub fn fixed(v: Vec<MetaVal>) -> Self {
        Self::Fixed(Fixed::new(v))
    }

    pub fn raw(v: Vec<Result<MetaVal, Error>>) -> Self {
        Self::Raw(Raw::new(v))
    }
}

impl<'a, I> Iterator for Producer<'a, I>
where
    I: Iterator<Item = Result<MetaVal, Error>>,
{
    type Item = Result<MetaVal, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            &mut Self::Source(ref mut s) => s.next(),
            &mut Self::Fixed(ref mut s) => s.next(),
            &mut Self::Raw(ref mut s) => s.next(),
            &mut Self::Flatten(ref mut s) => s.next(),
        }
    }
}
