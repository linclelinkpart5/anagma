//! Represents an argument that is fully realized, i.e. not a delayed expression.
//! These values need to be stable over time, as they can be used in partialled operators.

use std::convert::TryFrom;

use crate::metadata::types::MetaVal;
use crate::updated_scripting::ops::Predicate;
use crate::updated_scripting::ops::Converter;

pub enum Arg {
    Value(MetaVal),
    Predicate(Predicate),
    Converter(Converter),

    // NOTE: Having a `Producer` here makes little sense.
    //       It would get exhausted and not be replenished!
    // Producer(Producer),
}

impl From<MetaVal> for Arg {
    fn from(mv: MetaVal) -> Self {
        Arg::Value(mv)
    }
}

impl TryFrom<Arg> for MetaVal {
    type Error = ();

    fn try_from(a: Arg) -> Result<Self, Self::Error> {
        match a {
            Arg::Value(mv) => Ok(mv),
            _ => Err(()),
        }
    }
}

impl TryFrom<Arg> for bool {
    type Error = ();

    fn try_from(a: Arg) -> Result<Self, Self::Error> {
        MetaVal::try_from(a).and_then(bool::try_from)
    }
}

impl TryFrom<Arg> for Vec<MetaVal> {
    type Error = ();

    fn try_from(a: Arg) -> Result<Self, Self::Error> {
        MetaVal::try_from(a).and_then(Vec::<MetaVal>::try_from)
    }
}
