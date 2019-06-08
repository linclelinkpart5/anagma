use std::convert::TryFrom;
use std::convert::TryInto;

use crate::metadata::types::MetaVal;
use crate::functions::Error;
use crate::functions::util::value_producer::ValueProducer;
use crate::functions::util::value_producer::Flatten;
use crate::functions::util::value_producer::Dedup;
use crate::functions::util::value_producer::Unique;
use crate::functions::util::value_producer::Filter;
use crate::functions::util::value_producer::Map;
use crate::functions::operand::Operand;
use crate::functions::util::NumberLike;
use crate::functions::util::UnaryPred;
use crate::functions::util::UnaryConv;

#[derive(Clone, Copy)]
enum MinMax { Min, Max, }

#[derive(Clone, Copy)]
enum RevSort { Rev, Sort, }

#[derive(Clone, Copy)]
enum SumProd { Sum, Prod, }

pub enum IterableLike<'il> {
    Sequence(Vec<MetaVal<'il>>),
    Producer(ValueProducer<'il>),
}

impl<'il> IterableLike<'il> {
    pub fn is_lazy(&self) -> bool {
        match self {
            &Self::Sequence(..) => false,
            &Self::Producer(..) => true,
        }
    }

    pub fn is_eager(&self) -> bool {
        !self.is_lazy()
    }

    pub fn count(self) -> Result<usize, Error> {
        match self {
            Self::Sequence(s) => Ok(s.len()),
            Self::Producer(p) => {
                let mut c: usize = 0;
                for res_mv in p { res_mv?; c += 1; }
                Ok(c)
            },
        }
    }

    pub fn first(self) -> Result<MetaVal<'il>, Error> {
        match self {
            Self::Sequence(s) => s.into_iter().next().ok_or(Error::EmptySequence),
            Self::Producer(p) => p.into_iter().next().ok_or(Error::EmptyProducer)?,
        }
    }

    pub fn last(self) -> Result<MetaVal<'il>, Error> {
        match self {
            Self::Sequence(s) => s.into_iter().last().ok_or(Error::EmptySequence),
            Self::Producer(p) => {
                let mut last = None;
                for res_mv in p { last = Some(res_mv?); }
                last.ok_or(Error::EmptyProducer)
            },
        }
    }

    fn min_in_max_in<'a, EF: FnOnce() -> Error>(vp: ValueProducer<'a>, flag: MinMax, ef: EF) -> Result<NumberLike, Error> {
        let mut vp = vp.into_iter();
        match vp.next() {
            None => Err(ef()),
            Some(first_res_mv) => {
                let mut target_nl: NumberLike = first_res_mv?.try_into()?;

                for res_mv in vp {
                    let nl: NumberLike = res_mv?.try_into()?;
                    target_nl = match flag {
                        MinMax::Min => target_nl.val_min(nl),
                        MinMax::Max => target_nl.val_max(nl),
                    };
                }

                Ok(target_nl)
            }
        }
    }

    // pub fn min_in(self) -> Result<NumberLike, Error> {
    // pub fn max_in(self) -> Result<NumberLike, Error> {
    // pub fn rev(self) -> Vec<MetaVal> {
    // pub fn sort(self) -> Vec<MetaVal> {
    // pub fn sum(self) -> Result<NumberLike, Error> {
    // pub fn prod(self) -> Result<NumberLike, Error> {

    pub fn flatten(self) -> Self {
        match self {
            Self::Sequence(s) => Self::Sequence(Flatten::new(s.into()).collect::<Result<Vec<_>, _>>().unwrap()),
            Self::Producer(p) => Self::Producer(ValueProducer::Flatten(Flatten::new(p))),
        }
    }

    pub fn dedup(self) -> Self {
        match self {
            Self::Sequence(s) => Self::Sequence(Dedup::new(s.into()).collect::<Result<Vec<_>, _>>().unwrap()),
            Self::Producer(p) => Self::Producer(ValueProducer::Dedup(Dedup::new(p))),
        }
    }

    pub fn unique(self) -> Self {
        match self {
            Self::Sequence(s) => Self::Sequence(Unique::new(s.into()).collect::<Result<Vec<_>, _>>().unwrap()),
            Self::Producer(p) => Self::Producer(ValueProducer::Unique(Unique::new(p))),
        }
    }
}

impl<'il> From<IterableLike<'il>> for Operand<'il> {
    fn from(il: IterableLike<'il>) -> Self {
        match il {
            IterableLike::Sequence(sequence) => Self::Value(MetaVal::Seq(sequence)),
            IterableLike::Producer(producer) => Self::Producer(producer),
        }
    }
}

impl<'il> TryFrom<Operand<'il>> for IterableLike<'il> {
    type Error = Error;

    fn try_from(value: Operand<'il>) -> Result<Self, Self::Error> {
        match value {
            Operand::Value(mv) => Self::try_from(mv),
            Operand::Producer(s) => Ok(Self::Producer(s)),
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
    type IntoIter = IteratorLike<'il>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Sequence(s) => IteratorLike::Sequence(s.into_iter()),
            Self::Producer(s) => IteratorLike::Producer(s),
        }
    }
}

pub enum IteratorLike<'il> {
    Sequence(std::vec::IntoIter<MetaVal<'il>>),
    Producer(ValueProducer<'il>),
}

impl<'il> Iterator for IteratorLike<'il> {
    type Item = Result<MetaVal<'il>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            &mut Self::Sequence(ref mut it) => it.next().map(Result::Ok),
            &mut Self::Producer(ref mut it) => it.next(),
        }
    }
}
