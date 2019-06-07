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
// use crate::functions::util::value_producer::ValueProducer;

#[derive(Clone, Copy, Debug)]
pub enum Op {
    Nth,
    StepBy,
    Chain,
    Zip,
    Map,
    Filter,
    SkipWhile,
    TakeWhile,
    Skip,
    Take,
    All,
    Any,
    Find,
    Position,
    Interleave,
    Intersperse,
    Chunks,
    Windows,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl Op {
    pub fn process<'a>(&self, o_a: Operand<'a>, o_b: Operand<'a>) -> Result<Operand<'a>, Error> {
        match self {
            &Self::Nth => {
                match o_a.try_into()? {
                    IterableLike::Sequence(s) => Impl::nth_s(s, o_b.try_into()?),
                    IterableLike::Producer(p) => Impl::nth(p, o_b.try_into()?),
                }.map(Operand::Value)
            },
            _ => Ok(Operand::Value(MetaVal::Nil)),
        }
    }
}
