use std::convert::TryFrom;
use std::convert::TryInto;

use crate::metadata::types::MetaVal;
use crate::scripting::Error;
use crate::scripting::expr::Expr;
use crate::scripting::util::value_producer::ValueProducer;
use crate::scripting::util::number_like::NumberLike;
use crate::scripting::util::UnaryPred;
use crate::scripting::util::UnaryConv;

/// Values that are pushed onto an arg stack.
/// In order for a stack to be valid, it must result in exactly one value arg after processing.
pub enum Arg<'o> {
    Producer(ValueProducer<'o>),
    Value(MetaVal),
    Usize(usize),
    UnaryPred(UnaryPred),
    UnaryConv(UnaryConv),
    Expr(Box<Expr<'o>>),
}

impl<'o> TryFrom<Arg<'o>> for ValueProducer<'o> {
    type Error = Error;

    fn try_from(o: Arg<'o>) -> Result<Self, Self::Error> {
        match o {
            Arg::Producer(vp) => Ok(vp),
            _ => Err(Error::NotProducer),
        }
    }
}

impl<'o> TryFrom<Arg<'o>> for usize {
    type Error = Error;

    fn try_from(o: Arg<'o>) -> Result<Self, Self::Error> {
        match o {
            Arg::Usize(u) => Ok(u),
            Arg::Value(MetaVal::Int(i)) => {
                if i < 0 { Err(Error::NotUsize) }
                else { Ok(i as usize) }
            },
            _ => Err(Error::NotUsize),
        }
    }
}

impl<'o> TryFrom<Arg<'o>> for bool {
    type Error = Error;

    fn try_from(o: Arg<'o>) -> Result<Self, Self::Error> {
        match o {
            Arg::Value(MetaVal::Bul(b)) => Ok(b),
            _ => Err(Error::NotBoolean),
        }
    }
}

impl<'o> TryFrom<Arg<'o>> for UnaryPred {
    type Error = Error;

    fn try_from(o: Arg<'o>) -> Result<Self, Self::Error> {
        match o {
            Arg::UnaryPred(p) => Ok(p),
            _ => Err(Error::NotPredicate),
        }
    }
}

impl<'o> TryFrom<Arg<'o>> for UnaryConv {
    type Error = Error;

    fn try_from(o: Arg<'o>) -> Result<Self, Self::Error> {
        match o {
            Arg::UnaryConv(c) => Ok(c),
            _ => Err(Error::NotConverter),
        }
    }
}

impl<'o> From<usize> for Arg<'o> {
    fn from(u: usize) -> Self {
        Arg::Usize(u)
    }
}

impl<'o, I> From<I> for Arg<'o>
where
    I: Into<MetaVal>,
{
    fn from(i: I) -> Self {
        Arg::Value(i.into())
    }
}

impl<'o> TryFrom<&Arg<'o>> for NumberLike {
    type Error = Error;

    fn try_from(o: &Arg<'o>) -> Result<Self, Self::Error> {
        match o {
            Arg::Value(ref v) => v.try_into(),
            _ => Err(Error::NotBoolean),
        }
    }
}
