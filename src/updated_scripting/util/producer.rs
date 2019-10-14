mod producers;

use std::convert::TryFrom;

use crate::metadata::types::MetaVal;
use crate::updated_scripting::Error;

pub use self::producers::*;

pub enum Producer<'a> {
    Source(Source<'a>),
    Fixed(Fixed),
    Raw(Raw),
}

impl<'a> From<Vec<MetaVal>> for Producer<'a> {
    fn from(v: Vec<MetaVal>) -> Self {
        Self::Fixed(v.into())
    }
}

impl<'a> From<Vec<Result<MetaVal, Error>>> for Producer<'a> {
    fn from(v: Vec<Result<MetaVal, Error>>) -> Self {
        Self::Raw(v.into())
    }
}

impl<'a> TryFrom<Producer<'a>> for Vec<MetaVal> {
    type Error = Error;

    fn try_from(vp: Producer<'a>) -> Result<Self, Self::Error> {
        vp.collect::<Result<Vec<_>, _>>()
    }
}

impl<'a> From<Producer<'a>> for Vec<Result<MetaVal, Error>> {
    fn from(vp: Producer<'a>) -> Self {
        vp.collect()
    }
}

impl<'a> Producer<'a> {
    pub fn fixed(v: Vec<MetaVal>) -> Self {
        Self::Fixed(Fixed::new(v))
    }

    pub fn raw(v: Vec<Result<MetaVal, Error>>) -> Self {
        Self::Raw(Raw::new(v))
    }
}

impl<'a> Iterator for Producer<'a> {
    type Item = Result<MetaVal, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            &mut Self::Source(ref mut s) => s.next(),
            &mut Self::Fixed(ref mut s) => s.next(),
            &mut Self::Raw(ref mut s) => s.next(),
        }
    }
}
