mod producers;

use std::convert::TryFrom;

use crate::metadata::types::MetaVal;
use crate::updated_scripting::Error;

pub use self::producers::*;

pub struct Producer(Box<dyn Iterator<Item = Result<MetaVal, Error>>>);

impl From<Vec<MetaVal>> for Producer {
    fn from(v: Vec<MetaVal>) -> Self {
        Self(Box::new(Fixed::new(v)))
    }
}

impl From<Vec<Result<MetaVal, Error>>> for Producer {
    fn from(v: Vec<Result<MetaVal, Error>>) -> Self {
        Self(Box::new(Raw::new(v)))
    }
}

impl TryFrom<Producer> for Vec<MetaVal> {
    type Error = Error;

    fn try_from(prod: Producer) -> Result<Self, Self::Error> {
        prod.0.collect::<Result<Vec<_>, _>>()
    }
}

impl From<Producer> for Vec<Result<MetaVal, Error>> {
    fn from(prod: Producer) -> Self {
        prod.0.collect()
    }
}

impl Iterator for Producer {
    type Item = Result<MetaVal, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}
