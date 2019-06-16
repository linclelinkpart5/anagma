use std::borrow::Cow;
use std::convert::TryFrom;
use std::convert::TryInto;

use crate::metadata::types::MetaVal;
use crate::scripting::Error;
use crate::scripting::util::value_producer::ValueProducer;

/// Represents one of several different kinds of iterables, producing "references" to meta values.
pub enum RefIterableLike<'il> {
    Sequence(Vec<Cow<'il, MetaVal>>),
    Producer(ValueProducer<'il>),
}

impl<'il> IntoIterator for RefIterableLike<'il> {
    type Item = Result<Cow<'il, MetaVal>, Error>;
    type IntoIter = RefIteratorLike<'il>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Sequence(s) => RefIteratorLike::Sequence(s.into_iter()),
            Self::Producer(s) => RefIteratorLike::Producer(s),
        }
    }
}

pub enum RefIteratorLike<'il> {
    Sequence(std::vec::IntoIter<Cow<'il, MetaVal>>),
    Producer(ValueProducer<'il>),
}

impl<'il> Iterator for RefIteratorLike<'il> {
    type Item = Result<Cow<'il, MetaVal>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            &mut Self::Sequence(ref mut it) => it.next().map(Result::Ok),
            &mut Self::Producer(ref mut it) => it.next().map(|res| res.map(Cow::Owned)),
        }
    }
}
