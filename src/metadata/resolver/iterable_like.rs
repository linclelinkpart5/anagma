///! Wrapper type for items on the consumer stack that behave as a sequence of meta values.

use crate::metadata::resolver::ops::Operand;
use crate::metadata::resolver::streams::Stream;
use crate::metadata::types::MetaVal;
use crate::metadata::resolver::Error;

pub enum IterableLike<'il> {
    Stream(Stream<'il>),
    Sequence(Vec<MetaVal<'il>>),
}

impl<'il> From<IterableLike<'il>> for Operand<'il> {
    fn from(il: IterableLike<'il>) -> Self {
        match il {
            IterableLike::Stream(stream) => Self::Stream(stream),
            IterableLike::Sequence(sequence) => Self::Value(MetaVal::Seq(sequence)),
        }
    }
}

impl<'il> IntoIterator for IterableLike<'il> {
    type Item = Result<MetaVal<'il>, Error>;
    type IntoIter = IterLike<'il>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Stream(s) => IterLike::Stream(s),
            Self::Sequence(s) => IterLike::Sequence(s.into_iter()),
        }
    }
}

pub enum IterLike<'il> {
    Stream(Stream<'il>),
    Sequence(std::vec::IntoIter<MetaVal<'il>>),
}

impl<'il> Iterator for IterLike<'il> {
    type Item = Result<MetaVal<'il>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            &mut Self::Stream(ref mut s) => s.next(),
            &mut Self::Sequence(ref mut s) => s.next().map(Result::Ok),
        }
    }
}
