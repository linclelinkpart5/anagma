pub mod op;
pub mod arg;

pub use self::arg::Arg;
pub use self::op::UnaryOp;
pub use self::op::BinaryOp;

use crate::functions::Error;

pub enum Expr<'e> {
    Unary(UnaryOp, Thunk<'e>),
    Binary(BinaryOp, Thunk<'e>, Thunk<'e>),
}

impl<'e> Expr<'e> {
    pub fn eval(self) -> Result<Arg<'e>, Error> {
        match self {
            Self::Unary(u_op, th) => u_op.process(th.eval()?),
            Self::Binary(b_op, th_a, th_b) => b_op.process(th_a.eval()?, th_b.eval()?),
        }
    }
}

pub enum Thunk<'t> {
    Arg(Arg<'t>),
    Expr(Box<Expr<'t>>),
}

impl<'t> Thunk<'t> {
    pub fn eval(self) -> Result<Arg<'t>, Error> {
        match self {
            Self::Arg(o) => Ok(o),
            Self::Expr(e) => e.eval(),
        }
    }
}
