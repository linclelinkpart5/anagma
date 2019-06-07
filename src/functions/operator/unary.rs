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
            &Self::Collect => Impl::collect(o.try_into()?)
                .map(MetaVal::Seq)
                .map(Operand::Value)
            ,
            &Self::Count => {
                match o.try_into()? {
                    IterableLike::Sequence(s) => Ok(Impl::count_s(s)),
                    IterableLike::Producer(p) => Impl::count(p),
                }.map(Operand::Usize)
            },
            &Self::First => {
                match o.try_into()? {
                    IterableLike::Sequence(s) => Impl::first_s(s),
                    IterableLike::Producer(p) => Impl::first(p),
                }.map(Operand::Value)
            },
            &Self::Last => {
                match o.try_into()? {
                    IterableLike::Sequence(s) => Impl::last_s(s),
                    IterableLike::Producer(p) => Impl::last(p),
                }.map(Operand::Value)
            },
            &Self::MinIn => {
                match o.try_into()? {
                    IterableLike::Sequence(s) => Impl::min_in_s(s),
                    IterableLike::Producer(p) => Impl::min_in(p),
                }.map(MetaVal::from).map(Operand::Value)
            },
            &Self::MaxIn => {
                match o.try_into()? {
                    IterableLike::Sequence(s) => Impl::max_in_s(s),
                    IterableLike::Producer(p) => Impl::max_in(p),
                }.map(MetaVal::from).map(Operand::Value)
            },
            &Self::Rev => {
                match o.try_into()? {
                    IterableLike::Sequence(s) => Ok(Impl::rev_s(s)),
                    IterableLike::Producer(p) => Impl::rev(p),
                }.map(MetaVal::Seq).map(Operand::Value)
            },
            &Self::Sort => {
                match o.try_into()? {
                    IterableLike::Sequence(s) => Ok(Impl::sort_s(s)),
                    IterableLike::Producer(p) => Impl::sort(p),
                }.map(MetaVal::Seq).map(Operand::Value)
            },
            &Self::Sum => {
                match o.try_into()? {
                    IterableLike::Sequence(s) => Impl::sum_s(s),
                    IterableLike::Producer(p) => Impl::sum(p),
                }.map(MetaVal::from).map(Operand::Value)
            },
            &Self::Prod => {
                match o.try_into()? {
                    IterableLike::Sequence(s) => Impl::prod_s(s),
                    IterableLike::Producer(p) => Impl::prod(p),
                }.map(MetaVal::from).map(Operand::Value)
            },
            &Self::AllEqual => {
                match o.try_into()? {
                    IterableLike::Sequence(ref s) => Ok(Impl::all_equal_rs(s)),
                    IterableLike::Producer(p) => Impl::all_equal(p),
                }.map(MetaVal::Bul).map(Operand::Value)
            },
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
