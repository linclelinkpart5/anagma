use std::borrow::Cow;

use crate::metadata::types::MetaVal;
use crate::new_scripting::Error;

/// Represents a generic iterable over metadata values, both owned and borrowed.
pub enum GenericIterable<'gi> {
    Vector(Vec<MetaVal>),
    Slice(&'gi [MetaVal]),
}

impl<'gi> GenericIterable<'gi> {
    pub fn is_lazy(&self) -> bool {
        match self {
            &GenericIterable::Vector(..) => false,
            &GenericIterable::Slice(..) => false,
        }
    }

    pub fn is_eager(&self) -> bool {
        !self.is_lazy()
    }

    pub fn collect(self) -> Result<Vec<MetaVal>, Error> {
        match self {
            GenericIterable::Vector(v) => Ok(v),
            GenericIterable::Slice(s) => Ok(s.to_vec()),
        }
    }

    pub fn count(self) -> Result<usize, Error> {
        match self {
            GenericIterable::Vector(v) => Ok(v.len()),
            GenericIterable::Slice(s) => Ok(s.len()),
        }
    }
}

impl<'gi> IntoIterator for GenericIterable<'gi> {
    type Item = Result<Cow<'gi, MetaVal>, Error>;
    type IntoIter = GenericIterator<'gi>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            GenericIterable::Vector(v) => GenericIterator::Vector(v.into_iter()),
            GenericIterable::Slice(s) => GenericIterator::Slice(s.iter()),
        }
    }
}

pub enum GenericIterator<'gi> {
    Vector(std::vec::IntoIter<MetaVal>),
    Slice(std::slice::Iter<'gi, MetaVal>),
}

impl<'gi> Iterator for GenericIterator<'gi> {
    type Item = Result<Cow<'gi, MetaVal>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            &mut GenericIterator::Vector(ref mut it) => it.next().map(Cow::Owned).map(Result::Ok),
            &mut GenericIterator::Slice(ref mut it) => it.next().map(Cow::Borrowed).map(Result::Ok),
        }
    }
}
