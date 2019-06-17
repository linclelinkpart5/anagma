use std::borrow::Cow;
use std::convert::TryFrom;

use crate::metadata::types::MetaVal;
use crate::scripting::Error;
use crate::scripting::util::value_producer::ValueProducer;
use crate::scripting::util::UnaryPred;

/// Represents one of several different kinds of iterables, producing "references" to meta values.
pub enum RefIterableLike<'il> {
    // Sequence(Vec<MetaVal>),
    RefSequence(&'il [MetaVal]),
    Producer(ValueProducer<'il>),
}

impl<'il> TryFrom<&'il MetaVal> for RefIterableLike<'il> {
    type Error = Error;

    fn try_from(value: &'il MetaVal) -> Result<Self, Self::Error> {
        match value {
            &MetaVal::Seq(ref s) => Ok(Self::RefSequence(s.as_slice())),
            _ => Err(Error::NotIterable),
        }
    }
}

impl<'il> From<&'il Vec<MetaVal>> for RefIterableLike<'il> {
    fn from(s: &'il Vec<MetaVal>) -> Self {
        RefIterableLike::RefSequence(s)
    }
}

impl<'il> From<ValueProducer<'il>> for RefIterableLike<'il> {
    fn from(p: ValueProducer<'il>) -> Self {
        RefIterableLike::Producer(p)
    }
}

impl<'il> IntoIterator for RefIterableLike<'il> {
    type Item = Result<Cow<'il, MetaVal>, Error>;
    type IntoIter = RefIteratorLike<'il>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            // Self::Sequence(s) => RefIteratorLike::Sequence(s.into_iter()),
            Self::RefSequence(s) => RefIteratorLike::RefSequence(s.into_iter()),
            Self::Producer(p) => RefIteratorLike::Producer(p),
        }
    }
}

pub enum RefIteratorLike<'il> {
    // Sequence(std::vec::IntoIter<MetaVal>),
    RefSequence(std::slice::Iter<'il, MetaVal>),
    Producer(ValueProducer<'il>),
}

impl<'il> Iterator for RefIteratorLike<'il> {
    type Item = Result<Cow<'il, MetaVal>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            // &mut Self::Sequence(ref mut it) => it.next().map(Cow::Owned).map(Result::Ok),
            &mut Self::RefSequence(ref mut it) => it.next().map(Cow::Borrowed).map(Result::Ok),
            &mut Self::Producer(ref mut it) => it.next().map(|res| res.map(Cow::Owned)),
        }
    }
}

#[derive(Clone, Copy)]
enum AllAny { All, Any, }

impl AllAny {
    fn target(self) -> bool {
        match self {
            Self::All => false,
            Self::Any => true,
        }
    }
}

impl<'il> RefIterableLike<'il> {
    pub fn all_equal(self) -> Result<bool, Error> {
        let mut it = self.into_iter();
        Ok(match it.next() {
            None => true,
            Some(res_first_mv) => {
                let first_mv = res_first_mv?;
                for res_mv in it {
                    let mv = res_mv?;
                    if mv != first_mv { return Ok(false) }
                }

                true
            },
        })
    }

    fn all_any(self, u_pred: UnaryPred, flag: AllAny) -> Result<bool, Error> {
        let target = flag.target();
        for res_mv in self {
            let mv = res_mv?;
            if u_pred(&mv)? == target { return Ok(target) }
        }

        Ok(!target)
    }

    pub fn all(self, u_pred: UnaryPred) -> Result<bool, Error> {
        self.all_any(u_pred, AllAny::All)
    }

    pub fn any(self, u_pred: UnaryPred) -> Result<bool, Error> {
        self.all_any(u_pred, AllAny::Any)
    }
}
