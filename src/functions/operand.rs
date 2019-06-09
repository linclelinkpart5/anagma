use std::convert::TryFrom;

use crate::metadata::types::MetaVal;
use crate::functions::Error;
use crate::functions::util::value_producer::ValueProducer;

/// Values that are pushed onto an operand stack.
/// In order for a stack to be valid, it must result in exactly one value operand after processing.
pub enum Operand<'o> {
    Producer(ValueProducer<'o>),
    Value(MetaVal<'o>),
    Usize(usize),
}

impl<'o> TryFrom<Operand<'o>> for ValueProducer<'o> {
    type Error = Error;

    fn try_from(o: Operand<'o>) -> Result<Self, Self::Error> {
        match o {
            Operand::Producer(vp) => Ok(vp),
            _ => Err(Error::NotProducer),
        }
    }
}

impl<'o> TryFrom<Operand<'o>> for usize {
    type Error = Error;

    fn try_from(o: Operand<'o>) -> Result<Self, Self::Error> {
        match o {
            Operand::Usize(u) => Ok(u),
            Operand::Value(MetaVal::Int(i)) => {
                if i < 0 { Err(Error::NotUsize) }
                else { Ok(i as usize) }
            },
            _ => Err(Error::NotUsize),
        }
    }
}

impl<'o> From<usize> for Operand<'o> {
    fn from(u: usize) -> Self {
        Operand::Usize(u)
    }
}

impl<'o, I> From<I> for Operand<'o>
where
    I: Into<MetaVal<'o>>,
{
    fn from(i: I) -> Self {
        Operand::Value(i.into())
    }
}
