use std::borrow::Cow;
use std::convert::TryInto;

use crate::metadata::types::MetaVal;
use crate::functions::util::StreamAdaptor;
use crate::functions::op::Error;

/// Values that are pushed onto an operand stack.
/// In order for a stack to be valid, it must result in exactly one value operand after processing.
#[derive(Debug)]
pub enum Operand<'o> {
    StreamAdaptor(StreamAdaptor<'o>),
    Value(Cow<'o, MetaVal<'o>>),
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
