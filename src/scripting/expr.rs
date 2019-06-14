pub mod op;
pub mod arg;
pub mod thunk;

pub use self::arg::Arg;
pub use self::op::UnaryOp;
pub use self::op::BinaryOp;
pub use self::thunk::Thunk;

use std::convert::TryFrom;
use std::convert::TryInto;

use crate::scripting::Error;

pub enum Expr<'e> {
    Unary(UnaryOp, Thunk<'e>),
    Binary(BinaryOp, Thunk<'e>, Thunk<'e>),
}

impl<'e> TryFrom<Expr<'e>> for Arg<'e> {
    type Error = Error;

    fn try_from(e: Expr<'e>) -> Result<Self, Self::Error> {
        match e {
            Expr::Unary(u_op, th) => u_op.process(th.try_into()?),
            Expr::Binary(b_op, th_a, th_b) => b_op.process(th_a.try_into()?, th_b.try_into()?),
        }
    }
}
