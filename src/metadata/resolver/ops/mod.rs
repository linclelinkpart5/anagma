pub mod nullary;
pub mod unary;
pub mod binary;

use metadata::resolver::streams::Stream;
use metadata::types::MetaVal;
use metadata::types::MetaKey;
use metadata::types::MetaKeyPath;
use metadata::resolver::iterable_like::IterableLike;
use metadata::resolver::number_like::NumberLike;
use metadata::resolver::context::ResolverContext;
use metadata::resolver::Error;
use metadata::resolver::ops::nullary::NullaryOp;
use metadata::resolver::ops::unary::UnaryOp;
use metadata::stream::block::FileMetaBlockStream;
use metadata::stream::value::MetaValueStream;
use util::file_walkers::ParentFileWalker;
use util::file_walkers::ChildFileWalker;

/// Values that are pushed onto an operand stack.
/// In order for a stack to be valid, it must result in exactly one value operand after processing.
#[derive(Debug)]
pub enum Operand<'o> {
    Stream(Stream<'o>),
    Value(MetaVal<'o>),
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

    pub fn pop(&mut self) -> Result<Operand, Error> {
        self.0.pop().ok_or_else(|| Error::EmptyStack)
    }

    pub fn push(&mut self, op: Operand<'o>) -> () {
        self.0.push(op)
    }

    pub fn pop_iterable_like(&mut self) -> Result<IterableLike, Error> {
        match self.pop()? {
            Operand::Stream(s) => Ok(IterableLike::Stream(s)),
            Operand::Value(MetaVal::Seq(s)) => Ok(IterableLike::Sequence(s)),
            _ => Err(Error::UnexpectedOperand),
        }
    }

    pub fn pop_number_like(&mut self) -> Result<NumberLike, Error> {
        match self.pop()? {
            Operand::Value(MetaVal::Int(i)) => Ok(NumberLike::Integer(i)),
            Operand::Value(MetaVal::Dec(d)) => Ok(NumberLike::Decimal(d)),
            _ => Err(Error::UnexpectedOperand),
        }
    }

    pub fn pop_key_path_like(&mut self) -> Result<MetaKeyPath, Error> {
        let it_like = match self.pop()? {
            Operand::Stream(s) => IterableLike::Stream(s),
            Operand::Value(MetaVal::Seq(s)) => IterableLike::Sequence(s),
            Operand::Value(MetaVal::Str(s)) => {
                // Special case, handle and return.
                return Ok(s.into());
            },
            _ => {
                return Err(Error::UnexpectedOperand);
            }
        };

        let mut mks: Vec<MetaKey> = vec![];

        for mv in it_like.into_iter() {
            match mv? {
                MetaVal::Str(s) => {
                    mks.push(s.into());
                },
                _ => return Err(Error::NotString),
            }
        }

        Ok(mks.into())
    }
}

pub enum Token<'o> {
    Operand(Operand<'o>),
    NullaryOp(NullaryOp),
    UnaryOp(UnaryOp),
    BinaryOp,
}

pub trait Op {
    fn process<'o>(&self, rc: &ResolverContext<'o>, stack: &mut OperandStack<'o>) -> Result<(), Error>;
}
