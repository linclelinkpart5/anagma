use crate::functions::Error;
use crate::functions::operand::Operand;

pub enum UnaryOp {
}

pub enum BinaryOp {
}

pub enum Expression<'e> {
    Unary(UnaryOp, Thunk<'e>),
    Binary(BinaryOp, Thunk<'e>, Thunk<'e>),
}

impl<'e> Expression<'e> {
    pub fn eval(self) -> Result<Operand<'e>, Error> {
        Err(Error::InvalidOperand)
    }
}

pub enum Thunk<'t> {
    Operand(Operand<'t>),
    Expression(Box<Expression<'t>>),
}

impl<'t> Thunk<'t> {
    pub fn eval(self) -> Result<Operand<'t>, Error> {
        match self {
            Self::Operand(o) => Ok(o),
            Self::Expression(e) => e.eval(),
        }
    }
}
