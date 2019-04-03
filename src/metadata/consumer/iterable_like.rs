///! Wrapper type for items on the consumer stack that behave as a sequence of meta values.

use metadata::consumer::ops::Operand;
use metadata::consumer::streams::Stream;
use metadata::types::MetaVal;

pub enum IterableLike<'k, 'p, 's> {
    Stream(Stream<'k, 'p, 's>),
    Sequence(Vec<MetaVal>),
}

impl<'k, 'p, 's> From<IterableLike<'k, 'p, 's>> for Operand<'k, 'p, 's> {
    fn from(il: IterableLike<'k, 'p, 's>) -> Self {
        match il {
            IterableLike::Stream(stream) => Self::Stream(stream),
            IterableLike::Sequence(sequence) => Self::Value(MetaVal::Seq(sequence)),
        }
    }
}

impl<'k, 'p, 's> IntoIterator for IterableLike<'k, 'p, 's> {
    type Item = MetaVal;
    type IntoIter = IterLike<'k, 'p, 's>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Stream(s) => IterLike::Stream(s),
            Self::Sequence(s) => IterLike::Sequence(s.into_iter()),
        }
    }
}

pub enum IterLike<'k, 'p, 's> {
    Stream(Stream<'k, 'p, 's>),
    Sequence(std::vec::IntoIter<MetaVal>),
}

impl<'k, 'p, 's> Iterator for IterLike<'k, 'p, 's> {
    type Item = MetaVal;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            &mut Self::Stream(ref mut s) => s.next(),
            &mut Self::Sequence(ref mut s) => s.next(),
        }
    }
}
