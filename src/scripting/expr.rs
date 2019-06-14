pub mod op;
pub mod arg;

pub use self::arg::Arg;
pub use self::op::UnaryOp;
pub use self::op::BinaryOp;

use std::convert::TryFrom;
use std::convert::TryInto;

use crate::scripting::Error;

pub enum Expr<'e> {
    Arg(Arg<'e>),
    Unary(UnaryOp, Box<Expr<'e>>),
    Binary(BinaryOp, Box<Expr<'e>>, Box<Expr<'e>>),
}

impl<'e> TryFrom<Expr<'e>> for Arg<'e> {
    type Error = Error;

    fn try_from(e: Expr<'e>) -> Result<Self, Self::Error> {
        match e {
            Expr::Arg(a) => Ok(a),
            Expr::Unary(u_op, e) => u_op.process((*e).try_into()?),
            Expr::Binary(b_op, e_a, e_b) => b_op.process((*e_a).try_into()?, (*e_b).try_into()?),
        }
    }
}

// Used for short-circuiting `and` and `or` operators.
impl<'e> TryFrom<Expr<'e>> for bool {
    type Error = Error;

    fn try_from(e: Expr<'e>) -> Result<Self, Self::Error> {
        Arg::try_from(e)?.try_into()
    }
}
