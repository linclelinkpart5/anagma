pub mod op;
pub mod arg;

pub use self::arg::Arg;
pub use self::op::Op1;
pub use self::op::Op2;

use std::convert::TryFrom;

use crate::util::Number;
use crate::scripting::Error;
use crate::scripting::expr::op::pred1::Pred1;
// use crate::scripting::util::UnaryPred;
use crate::scripting::util::UnaryConv;

pub enum Expr<'e> {
    Arg(Arg<'e>),
    Op1(Op1, Box<Expr<'e>>),
    Op2(Op2, Box<Expr<'e>>, Box<Expr<'e>>),
}

impl<'e> TryFrom<Expr<'e>> for Arg<'e> {
    type Error = Error;

    fn try_from(e: Expr<'e>) -> Result<Self, Self::Error> {
        match e {
            Expr::Arg(a) => Ok(a),
            Expr::Op1(u_op, e) => u_op.process(*e),
            Expr::Op2(b_op, e_a, e_b) => b_op.process(*e_a, *e_b),
        }
    }
}

// Used for short-circuiting `and` and `or` operators.
impl<'e> TryFrom<Expr<'e>> for bool {
    type Error = Error;

    fn try_from(e: Expr<'e>) -> Result<Self, Self::Error> {
        Arg::try_from(e).and_then(Self::try_from)
    }
}

impl<'e> TryFrom<Expr<'e>> for usize {
    type Error = Error;

    fn try_from(e: Expr<'e>) -> Result<Self, Self::Error> {
        Arg::try_from(e).and_then(Self::try_from)
    }
}

impl<'e> TryFrom<Expr<'e>> for Pred1 {
    type Error = Error;

    fn try_from(e: Expr<'e>) -> Result<Self, Self::Error> {
        Arg::try_from(e).and_then(Self::try_from)
    }
}

impl<'e> TryFrom<Expr<'e>> for UnaryConv {
    type Error = Error;

    fn try_from(e: Expr<'e>) -> Result<Self, Self::Error> {
        Arg::try_from(e).and_then(Self::try_from)
    }
}

impl<'e> From<Arg<'e>> for Expr<'e> {
    fn from(a: Arg<'e>) -> Self {
        Self::Arg(a)
    }
}

impl<'e> TryFrom<Expr<'e>> for Number {
    type Error = Error;

    fn try_from(expr: Expr<'e>) -> Result<Self, Self::Error> {
        Arg::try_from(expr).and_then(Number::try_from).map_err(|_| Error::NotNumeric)
    }
}
