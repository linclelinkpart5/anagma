pub mod source;
pub mod unary;
pub mod binary;
pub mod predicate;
pub mod partial;

use crate::metadata::resolver::streams::Stream;
use crate::metadata::types::MetaVal;
use crate::metadata::resolver::Error;
use crate::metadata::resolver::ops::source::Source;
use crate::metadata::resolver::ops::unary::UnaryOp;
use crate::metadata::resolver::ops::binary::BinaryOp;

/// Values that are pushed onto an operand stack.
/// In order for a stack to be valid, it must result in exactly one value operand after processing.
#[derive(Debug)]
pub enum Operand<'o> {
    Stream(Stream<'o>),
    Value(MetaVal<'o>),
    UnaryOp(UnaryOp),
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

    // pub fn pop_key_path_like(&mut self) -> Result<MetaKeyPath, Error> {
    //     let it_like = match self.pop()? {
    //         Operand::Stream(s) => IterableLike::Stream(s),
    //         Operand::Value(MetaVal::Seq(s)) => IterableLike::Sequence(s),
    //         Operand::Value(MetaVal::Str(s)) => {
    //             // Special case, handle and return.
    //             return Ok(s.into());
    //         },
    //         _ => {
    //             return Err(Error::UnexpectedOperand);
    //         }
    //     };

    //     let mut mks: Vec<MetaKey> = vec![];

    //     for mv in it_like.into_iter() {
    //         match mv? {
    //             MetaVal::Str(s) => {
    //                 mks.push(s.into());
    //             },
    //             _ => return Err(Error::NotString),
    //         }
    //     }

    //     Ok(mks.into())
    // }
}

pub enum Token<'o> {
    Operand(Operand<'o>),
    Source(Source),
    UnaryOp(UnaryOp),
    BinaryOp(BinaryOp),
}

pub trait Op {
    fn process_stack<'o>(&self, stack: &mut OperandStack<'o>) -> Result<(), Error>;
}
