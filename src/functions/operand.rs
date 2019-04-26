use std::borrow::Cow;

use crate::metadata::types::MetaVal;

#[derive(Debug)]
pub enum Error {
    EmptyStack,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::EmptyStack => write!(f, "empty operand stack"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Self::EmptyStack => None,
        }
    }
}

/// Values that are pushed onto an operand stack.
/// In order for a stack to be valid, it must result in exactly one value operand after processing.
#[derive(Debug)]
pub enum Operand<'o> {
    // Stream(Stream<'o>),
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
