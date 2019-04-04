use metadata::resolver::streams::Stream;
use metadata::types::MetaVal;
use metadata::resolver::iterable_like::IterableLike;
use metadata::resolver::number_like::NumberLike;

#[derive(Debug)]
pub enum Error {
    UnexpectedOperand,
    EmptyStack,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::UnexpectedOperand => write!(f, "unexpected operand on stack"),
            Self::EmptyStack => write!(f, "empty operand stack"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Self::UnexpectedOperand => None,
            Self::EmptyStack => None,
        }
    }
}

pub enum Operand<'k, 'p, 's> {
    Stream(Stream<'k, 'p, 's>),
    Value(MetaVal),
}

pub struct OperandStack<'k, 'p, 's>(Vec<Operand<'k, 'p, 's>>);

impl<'k, 'p, 's> OperandStack<'k, 'p, 's> {
    pub fn pop(&mut self) -> Result<Operand, Error> {
        self.0.pop().ok_or_else(|| Error::EmptyStack)
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
}

pub struct Expression<'k, 'p, 's> {
    source_stream: Stream<'k, 'p, 's>,
    tokens: Vec<Token<'k, 'p, 's>>,
}

pub enum Token<'k, 'p, 's> {
    Operand(Operand<'k, 'p, 's>),
    UnaryOp,
    BinaryOp,
}
