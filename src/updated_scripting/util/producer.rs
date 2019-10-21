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

impl Producer {
    pub fn new(iter: impl Iterator<Item = Result<MetaVal, Error>> + 'static) -> Self {
        Self(Box::new(iter))
    }

    pub fn collect(self) -> Result<Vec<MetaVal>, Error> {
        // NOTE: Need to define this weirdly since `.collect()` also exists on this struct.
        Iterator::collect::<Result<Vec<_>, _>>(self.into_iter())
    }

    pub fn len(self) -> Result<usize, Error> {
        let mut n = 0;
        for res_item in self { res_item?; n += 1; }
        Ok(n)
    }

    pub fn first(self) -> Result<Option<MetaVal>, Error> {
        self.into_iter().next().transpose()
    }

    pub fn last(self) -> Result<Option<MetaVal>, Error> {
        let mut last_seen = None;
        for res_item in self {
            let item = res_item?;
            last_seen.replace(item);
        }
        Ok(last_seen)
    }

    pub fn nth(self, n: usize) -> Result<Option<MetaVal>, Error> {
        let mut iter = self;
        let mut i = 0;
        while i < n {
            iter.next().transpose()?;
            i += 1;
        }
        iter.next().transpose()
    }
}
