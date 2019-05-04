pub mod converter;
pub mod predicate;
pub mod iter_consumer;
pub mod iter_adaptor;

pub use self::converter::Converter;
pub use self::predicate::Predicate;
pub use self::iter_consumer::IterConsumer;
pub use self::iter_adaptor::IterAdaptor;

use std::convert::TryInto;
use std::convert::TryFrom;

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

impl From<IterConsumer> for Op {
    fn from(it_cons: IterConsumer) -> Self {
        match it_cons {
            IterConsumer::Collect => Self::Collect,
            IterConsumer::Count => Self::Count,
            IterConsumer::First => Self::First,
            IterConsumer::Last => Self::Last,
            IterConsumer::MaxIn => Self::MaxIn,
            IterConsumer::MinIn => Self::MinIn,
            IterConsumer::Rev => Self::Rev,
            IterConsumer::Sort => Self::Sort,
            IterConsumer::Sum => Self::Sum,
            IterConsumer::Prod => Self::Prod,
            IterConsumer::AllEqual => Self::AllEqual,
        }
    }
}

impl From<IterAdaptor> for Op {
    fn from(it_adap: IterAdaptor) -> Self {
        match it_adap {
            IterAdaptor::Flatten => Self::Flatten,
            IterAdaptor::Dedup => Self::Dedup,
            IterAdaptor::Unique => Self::Unique,
        }
    }
}

impl TryFrom<Op> for Predicate {
    type Error = Error;

    fn try_from(op: Op) -> Result<Self, Self::Error> {
        match op {
            Op::AllEqual => Ok(Self::AllEqual),
            _ => Err(Error::NotPredicate),
        }
    }
}

impl TryFrom<Op> for Converter {
    type Error = Error;

    fn try_from(op: Op) -> Result<Self, Self::Error> {
        // First, try to convert to predicate.
        let res_pred: Result<Predicate, _> = op.try_into();
        if let Ok(pred) = res_pred {
            Ok(Self::Predicate(pred))
        }
        else {
            match op {
                Op::Count => Ok(Self::Count),
                Op::First => Ok(Self::First),
                Op::Last => Ok(Self::Last),
                Op::MaxIn => Ok(Self::MaxIn),
                Op::MinIn => Ok(Self::MinIn),
                Op::Rev => Ok(Self::Rev),
                Op::Sort => Ok(Self::Sort),
                Op::Sum => Ok(Self::Sum),
                Op::Prod => Ok(Self::Prod),
                Op::Flatten => Ok(Self::Flatten),
                Op::Dedup => Ok(Self::Dedup),
                Op::Unique => Ok(Self::Unique),
                _ => Err(Error::NotConverter),
            }
        }
    }
}
