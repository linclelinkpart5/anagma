use std::convert::TryInto;

use crate::metadata::types::MetaVal;
use crate::functions::util::StreamAdaptor;
use crate::functions::Error;

/// Values that are pushed onto an operand stack.
/// In order for a stack to be valid, it must result in exactly one value operand after processing.
#[derive(Debug)]
pub enum Operand<'o> {
    StreamAdaptor(StreamAdaptor<'o>),
    Value(MetaVal<'o>),
}

impl<'o> TryInto<MetaVal<'o>> for Operand<'o> {
    type Error = Error;

    fn try_into(self) -> Result<MetaVal<'o>, Self::Error> {
        match self {
            Self::Value(mv) => Ok(mv),
            _ => Err(Error::InvalidOperand),
        }
    }
}

impl<'o> TryInto<StreamAdaptor<'o>> for Operand<'o> {
    type Error = Error;

    fn try_into(self) -> Result<StreamAdaptor<'o>, Self::Error> {
        match self {
            Self::StreamAdaptor(sa) => Ok(sa),
            _ => Err(Error::InvalidOperand),
        }
    }
}

#[derive(Debug)]
pub struct OperandStack<'o>(Vec<Operand<'o>>);

impl<'o> OperandStack<'o> {
    pub fn new() -> Self {
        OperandStack(vec![])
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn pop(&mut self) -> Result<Operand<'o>, Error> {
        self.0.pop().ok_or_else(|| Error::EmptyStack)
    }

    pub fn push(&mut self, op: Operand<'o>) -> () {
        self.0.push(op)
    }
}
