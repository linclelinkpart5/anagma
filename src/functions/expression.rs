use crate::functions::Error;
use crate::functions::operand::Operand;
use crate::functions::operator::UnaryOp;
use crate::functions::operator::BinaryOp;

pub enum Expression<'e> {
    Unary(UnaryOp, Thunk<'e>),
    Binary(BinaryOp, Thunk<'e>, Thunk<'e>),
}

impl<'e> Expression<'e> {
    pub fn eval(self) -> Result<Operand<'e>, Error> {
        match self {
            Self::Unary(u_op, th) => u_op.process(th.eval()?),
            Self::Binary(b_op, th_a, th_b) => b_op.process(th_a.eval()?, th_b.eval()?),
        }
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
