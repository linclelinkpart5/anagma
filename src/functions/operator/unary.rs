pub mod converter;
pub mod predicate;
pub mod iter_consumer;
pub mod iter_adaptor;

pub use self::converter::Converter;
pub use self::predicate::Predicate;
pub use self::iter_consumer::IterConsumer;
pub use self::iter_adaptor::IterAdaptor;

use std::convert::TryInto;

use crate::metadata::types::MetaVal;
use crate::functions::Error;
use crate::functions::util::StreamAdaptor;
use crate::functions::operand::Operand;

#[derive(Clone, Copy, Debug)]
pub enum OpImpl {
    Converter(Converter),
    IterConsumer(IterConsumer),
    IterAdaptor(IterAdaptor),
}

impl OpImpl {
    pub fn process<'o>(&self, operand: Operand<'o>) -> Result<Operand<'o>, Error> {
        match self {
            &Self::Converter(conv) => {
                let mv: MetaVal<'_> = operand.try_into()?;
                conv.process(mv).map(Operand::Value)
            },
            &Self::IterConsumer(ic) => {
                let sa: StreamAdaptor<'_> = operand.try_into()?;
                ic.process(sa).map(Operand::Value)
            },
            &Self::IterAdaptor(ia) => {
                let sa: StreamAdaptor<'_> = operand.try_into()?;
                ia.process(sa).map(Operand::StreamAdaptor)
            },
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Op {
    Count,
    First,
    Last,
    MaxIn,
    MinIn,
    Rev,
    Sort,
    Sum,
    Prod,
    Flatten,
    Dedup,
    Unique,
    Collect,
    AllEqual,
}

impl From<Predicate> for Op {
    fn from(pred: Predicate) -> Self {
        match pred {
            Predicate::AllEqual => Self::AllEqual,
        }
    }
}

impl From<Converter> for Op {
    fn from(conv: Converter) -> Self {
        match conv {
            Converter::Count => Self::Count,
            Converter::First => Self::First,
            Converter::Last => Self::Last,
            Converter::MaxIn => Self::MaxIn,
            Converter::MinIn => Self::MinIn,
            Converter::Rev => Self::Rev,
            Converter::Sort => Self::Sort,
            Converter::Sum => Self::Sum,
            Converter::Prod => Self::Prod,
            Converter::Flatten => Self::Flatten,
            Converter::Dedup => Self::Dedup,
            Converter::Unique => Self::Unique,
            Converter::Predicate(pred) => pred.into(),
        }
    }
}
