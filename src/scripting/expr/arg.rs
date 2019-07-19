use std::convert::TryFrom;
use std::convert::TryInto;

use crate::metadata::types::MetaVal;
use crate::scripting::Error;
use crate::scripting::util::value_producer::ValueProducer;
use crate::util::Number;
use crate::scripting::expr::op::pred1::Pred1;
// use crate::scripting::util::UnaryPred;
use crate::scripting::util::UnaryConv;

/// Values that are pushed onto an arg stack.
/// In order for a stack to be valid, it must result in exactly one value arg after processing.
pub enum Arg<'a> {
    Producer(ValueProducer<'a>),
    Value(MetaVal),
    Usize(usize),
    Pred1(Pred1),
    UnaryConv(UnaryConv),
}

impl<'a> TryFrom<Arg<'a>> for ValueProducer<'a> {
    type Error = Error;

    fn try_from(arg: Arg<'a>) -> Result<Self, Self::Error> {
        match arg {
            Arg::Producer(vp) => Ok(vp),
            _ => Err(Error::NotProducer),
        }
    }
}

impl<'a> TryFrom<Arg<'a>> for usize {
    type Error = Error;

    fn try_from(arg: Arg<'a>) -> Result<Self, Self::Error> {
        match arg {
            Arg::Usize(u) => Ok(u),
            Arg::Value(MetaVal::Int(i)) => {
                if i < 0 { Err(Error::NotUsize) }
                else { Ok(i as usize) }
            },
            _ => Err(Error::NotUsize),
        }
    }
}

impl<'a> TryFrom<Arg<'a>> for bool {
    type Error = Error;

    fn try_from(arg: Arg<'a>) -> Result<Self, Self::Error> {
        match arg {
            Arg::Value(MetaVal::Bul(b)) => Ok(b),
            _ => Err(Error::NotBoolean),
        }
    }
}

impl<'a> TryFrom<Arg<'a>> for Pred1 {
    type Error = Error;

    fn try_from(arg: Arg<'a>) -> Result<Self, Self::Error> {
        match arg {
            Arg::Pred1(p) => Ok(p),
            _ => Err(Error::NotPredicate),
        }
    }
}

impl<'a> TryFrom<Arg<'a>> for UnaryConv {
    type Error = Error;

    fn try_from(arg: Arg<'a>) -> Result<Self, Self::Error> {
        match arg {
            Arg::UnaryConv(c) => Ok(c),
            _ => Err(Error::NotConverter),
        }
    }
}

impl<'a> From<usize> for Arg<'a> {
    fn from(u: usize) -> Self {
        Arg::Usize(u)
    }
}

impl<'a, I> From<I> for Arg<'a>
where
    I: Into<MetaVal>,
{
    fn from(i: I) -> Self {
        Arg::Value(i.into())
    }
}

impl<'a> TryFrom<&Arg<'a>> for Number {
    type Error = Error;

    fn try_from(arg: &Arg<'a>) -> Result<Self, Self::Error> {
        match arg {
            Arg::Value(ref v) => v.try_into().map_err(|_| Error::NotNumeric),
            _ => Err(Error::NotNumeric),
        }
    }
}

impl<'a> TryFrom<Arg<'a>> for Number {
    type Error = Error;

    fn try_from(arg: Arg<'a>) -> Result<Self, Self::Error> {
        match arg {
            Arg::Value(mv) => mv.try_into().map_err(|_| Error::NotNumeric),
            _ => Err(Error::NotNumeric),
        }
    }
}

// NOTE: Superseded by blanket impl.
// impl<'k> From<Number> for Arg<'k> {
//     fn from(num: Number) -> Arg<'k> {
//         Arg::Value(num.into())
//     }
// }
