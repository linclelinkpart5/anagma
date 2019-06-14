pub mod op;
pub mod arg;
pub mod thunk;

pub use self::arg::Arg;
pub use self::op::UnaryOp;
pub use self::op::BinaryOp;
pub use self::thunk::Thunk;

use crate::scripting::Error;

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
