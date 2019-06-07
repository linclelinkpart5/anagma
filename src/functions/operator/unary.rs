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
}

impl Op {
    pub fn process<'a>(&self, o: Operand<'a>) -> Result<Operand<'a>, Error> {
        match self {
            &Self::Collect => Impl::collect(o.try_into()?)
                .map(MetaVal::Seq)
                .map(Operand::Value)
            ,
            &Self::Count => {
                let it: IterableLike = o.try_into()?;
                match it {
                    IterableLike::Sequence(s) => Ok(Impl::count_s(s)),
                    IterableLike::Producer(p) => Impl::count(p),
                }.map(Operand::Usize)
            },
            &Self::First => {
                let it: IterableLike = o.try_into()?;
                match it {
                    IterableLike::Sequence(s) => Impl::first_s(s),
                    IterableLike::Producer(p) => Impl::first(p),
                }.map(Operand::Value)
            },
            &Self::Last => {
                let it: IterableLike = o.try_into()?;
                match it {
                    IterableLike::Sequence(s) => Impl::last_s(s),
                    IterableLike::Producer(p) => Impl::last(p),
                }.map(Operand::Value)
            },
            &Self::MinIn => {
                let it: IterableLike = o.try_into()?;
                match it {
                    IterableLike::Sequence(s) => Impl::min_in_s(s),
                    IterableLike::Producer(p) => Impl::min_in(p),
                }.map(MetaVal::from).map(Operand::Value)
            },
            &Self::MaxIn => {
                let it: IterableLike = o.try_into()?;
                match it {
                    IterableLike::Sequence(s) => Impl::max_in_s(s),
                    IterableLike::Producer(p) => Impl::max_in(p),
                }.map(MetaVal::from).map(Operand::Value)
            },
            &Self::Rev => {
                let it: IterableLike = o.try_into()?;
                match it {
                    IterableLike::Sequence(s) => Impl::max_in_s(s),
                    IterableLike::Producer(p) => Impl::max_in(p),
                }.map(MetaVal::from).map(Operand::Value)
            },
            _ => Ok(Operand::Value(MetaVal::Nil)),
        }
    }
}
