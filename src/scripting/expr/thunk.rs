use std::convert::TryFrom;
use std::convert::TryInto;

use crate::scripting::expr::Expr;
use crate::scripting::expr::arg::Arg;
use crate::scripting::Error;

pub enum Thunk<'t> {
    Arg(Arg<'t>),
    Expr(Box<Expr<'t>>),
}

impl<'t> TryFrom<Thunk<'t>> for Arg<'t> {
    type Error = Error;

    fn try_from(t: Thunk<'t>) -> Result<Self, Self::Error> {
        match t {
            Thunk::Arg(o) => Ok(o),
            // LEARN: Why is the dereference needed?
            Thunk::Expr(e) => (*e).try_into(),
        }
    }
}

// Used for short-circuiting `and` and `or` operators.
impl<'t> TryFrom<Thunk<'t>> for bool {
    type Error = Error;

    fn try_from(t: Thunk<'t>) -> Result<Self, Self::Error> {
        Arg::try_from(t)?.try_into()
    }
}
