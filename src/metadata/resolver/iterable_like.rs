///! Wrapper type for items on the consumer stack that behave as a sequence of meta values.

use std::convert::TryFrom;

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

impl<'il> TryFrom<Operand<'il>> for IterableLike<'il> {
    type Error = Error;

    fn try_from(value: Operand<'il>) -> Result<Self, Self::Error> {
        match value {
            Operand::Stream(s) => Ok(Self::Stream(s)),
            Operand::Value(mv) => Self::try_from(mv),
            _ => Err(Error::NotIterable),
        }
    }
}

impl<'il> TryFrom<MetaVal<'il>> for IterableLike<'il> {
    type Error = Error;

    fn try_from(value: MetaVal<'il>) -> Result<Self, Self::Error> {
        match value {
            MetaVal::Seq(s) => Ok(Self::Sequence(s)),
            _ => Err(Error::NotIterable),
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
