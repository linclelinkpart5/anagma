
use std::borrow::Cow;

use crate::metadata::types::MetaVal;

pub enum IteratorLike<'a> {
    Slice(std::slice::Iter<'a, MetaVal>),
    Vector(std::vec::IntoIter<MetaVal>),
}

impl<'a> Iterator for IteratorLike<'a> {
    type Item = Cow<'a, MetaVal>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            &mut Self::Slice(ref mut it) => it.next().map(Cow::Borrowed),
            &mut Self::Vector(ref mut it) => it.next().map(Cow::Owned),
        }
    }
}
