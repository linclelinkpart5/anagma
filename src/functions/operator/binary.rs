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

use crate::metadata::types::MetaVal;
use crate::functions::Error;
use crate::functions::operand::Operand;
use crate::functions::util::iterable_like::IterableLike;
// use crate::functions::util::value_producer::ValueProducer;

#[derive(Clone, Copy, Debug)]
pub enum Op {
    Nth,
    All,
    Any,
    Find,
    Position,
    Filter,
    Map,
    StepBy,
    Chain,
    Zip,
    Skip,
    Take,
    SkipWhile,
    TakeWhile,
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
            &Self::Nth =>
                IterableLike::try_from(o_a)?.nth(o_b.try_into()?).map(Operand::from),
            &Self::All =>
                IterableLike::try_from(o_a)?.all(o_b.try_into()?).map(Operand::from),
            &Self::Any =>
                IterableLike::try_from(o_a)?.any(o_b.try_into()?).map(Operand::from),
            &Self::Find =>
                IterableLike::try_from(o_a)?.find(o_b.try_into()?).map(Operand::from),
            &Self::Position =>
                IterableLike::try_from(o_a)?.position(o_b.try_into()?).map(Operand::from),
            &Self::Filter =>
                IterableLike::try_from(o_a)?.filter(o_b.try_into()?).map(Operand::from),
            &Self::Map =>
                IterableLike::try_from(o_a)?.map(o_b.try_into()?).map(Operand::from),
            &Self::StepBy =>
                IterableLike::try_from(o_a)?.step_by(o_b.try_into()?).map(Operand::from),
            &Self::Chain =>
                IterableLike::try_from(o_a)?.chain(o_b.try_into()?).map(Operand::from),
            &Self::Zip =>
                IterableLike::try_from(o_a)?.zip(o_b.try_into()?).map(Operand::from),
            &Self::Skip =>
                IterableLike::try_from(o_a)?.skip(o_b.try_into()?).map(Operand::from),
            &Self::Take =>
                IterableLike::try_from(o_a)?.take(o_b.try_into()?).map(Operand::from),
            &Self::SkipWhile =>
                IterableLike::try_from(o_a)?.skip_while(o_b.try_into()?).map(Operand::from),
            &Self::TakeWhile =>
                IterableLike::try_from(o_a)?.take_while(o_b.try_into()?).map(Operand::from),
            _ => Ok(Operand::Value(MetaVal::Nil)),
        }
    }
}
