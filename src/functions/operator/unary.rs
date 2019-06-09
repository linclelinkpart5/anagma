pub mod converter;
pub mod predicate;
pub mod iter_consumer;
pub mod iter_adaptor;
pub mod imp;

pub use self::converter::Converter;
pub use self::predicate::Predicate;
pub use self::iter_consumer::IterConsumer;
pub use self::iter_adaptor::IterAdaptor;

use std::convert::TryInto;
use std::convert::TryFrom;

use self::imp::Impl;

use crate::metadata::types::MetaVal;
use crate::functions::Error;
use crate::functions::operand::Operand;
use crate::functions::util::iterable_like::IterableLike;
use crate::functions::util::value_producer::ValueProducer;

#[derive(Clone, Copy, Debug)]
pub enum Op {
    Collect,
    Count,
    First,
    Last,
    MinIn,
    MaxIn,
    Rev,
    Sort,
    Sum,
    Prod,
    AllEqual,
    Flatten,
    Dedup,
    Unique,
    Neg,
    Abs,
    Not,
}

impl Op {
    pub fn process<'a>(&self, o: Operand<'a>) -> Result<Operand<'a>, Error> {
        match self {
            &Self::Collect =>
                IterableLike::try_from(o)?.collect().map(Operand::from),
            &Self::Count =>
                IterableLike::try_from(o)?.count().map(Operand::from),
            &Self::First =>
                IterableLike::try_from(o)?.first().map(Operand::from),
            &Self::Last =>
                IterableLike::try_from(o)?.last().map(Operand::from),
            &Self::MinIn =>
                IterableLike::try_from(o)?.min_in().map(Operand::from),
            &Self::MaxIn =>
                IterableLike::try_from(o)?.max_in().map(Operand::from),
            &Self::Rev =>
                IterableLike::try_from(o)?.rev().map(Operand::from),
            &Self::Sort =>
                IterableLike::try_from(o)?.sort().map(Operand::from),
            &Self::Sum =>
                IterableLike::try_from(o)?.sum().map(Operand::from),
            &Self::Prod =>
                IterableLike::try_from(o)?.prod().map(Operand::from),
            &Self::AllEqual =>
                IterableLike::try_from(o)?.all_equal().map(Operand::from),
            &Self::Flatten => {
                match o.try_into()? {
                    IterableLike::Sequence(s) => Ok(Operand::Value(MetaVal::Seq(Impl::flatten_s(s)))),
                    IterableLike::Producer(p) => Ok(Operand::Producer(ValueProducer::Flatten(Impl::flatten(p)))),
                }
            },
            &Self::Dedup => {
                match o.try_into()? {
                    IterableLike::Sequence(s) => Ok(Operand::Value(MetaVal::Seq(Impl::dedup_s(s)))),
                    IterableLike::Producer(p) => Ok(Operand::Producer(ValueProducer::Dedup(Impl::dedup(p)))),
                }
            },
            &Self::Unique => {
                match o.try_into()? {
                    IterableLike::Sequence(s) => Ok(Operand::Value(MetaVal::Seq(Impl::unique_s(s)))),
                    IterableLike::Producer(p) => Ok(Operand::Producer(ValueProducer::Unique(Impl::unique(p)))),
                }
            },
            &Self::Neg => Ok(Impl::neg(o.try_into()?).into()),
            &Self::Abs => Ok(Impl::abs(o.try_into()?).into()),
            &Self::Not => {
                match o {
                    Operand::Value(MetaVal::Bul(b)) => Ok(Operand::Value(MetaVal::Bul(Impl::not(b)))),
                    _ => Err(Error::NotBoolean),
                }
            },
        }
    }
}
