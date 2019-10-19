
use std::borrow::Cow;

use crate::metadata::types::MetaVal;
use crate::updated_scripting::Error;
use crate::updated_scripting::util::Producer;

pub enum IteratorLike<'a> {
    Slice(std::slice::Iter<'a, MetaVal>),
    Vector(std::vec::IntoIter<MetaVal>),
    Producer(Producer),
}

impl<'a> Iterator for IteratorLike<'a> {
    type Item = Result<Cow<'a, MetaVal>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            &mut Self::Slice(ref mut it) => it.next().map(Cow::Borrowed).map(Result::Ok),
            &mut Self::Vector(ref mut it) => it.next().map(Cow::Owned).map(Result::Ok),
            &mut Self::Producer(ref mut it) => it.next().map(|res| res.map(Cow::Owned)),
        }
    }
}
