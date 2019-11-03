
use std::convert::TryFrom;
use std::cmp::Ordering;
use std::collections::BTreeMap;

use crate::util::Number;
use crate::metadata::types::MetaKey;
use crate::metadata::types::MetaVal;
use crate::updated_scripting::Error;
use crate::updated_scripting::util::IterableLike;
use crate::updated_scripting::arg::Arg;

pub enum Predicate {
    AllEqual,
    IsEmpty,
    Not,
    All(Box<Predicate>),
    Any(Box<Predicate>),
    And(bool),
    Or(bool),
    Xor(bool),
    Eq(Number),
    Ne(Number),
    Lt(Number),
    Le(Number),
    Gt(Number),
    Ge(Number),
    HasKeyA(MetaKey),
    HasKeyB(BTreeMap<MetaKey, MetaVal>),
}

impl Predicate {
    pub fn test(&self, arg: &Arg) -> Result<bool, Error> {
        match self {
            &Self::AllEqual => IterableLike::try_from(arg)?.all_equal(),
            &Self::IsEmpty => IterableLike::try_from(arg)?.is_empty(),
            &Self::Not => Ok(!bool::try_from(arg).map_err(|_| Error::NotBoolean)?),
            // &Self::All(ref pred) => {
            //     // TODO: Have `IterableLike::all()` accept this `Predicate` type and use it instead of trait.
            //     for v in IterableLike::try_from(arg)? {
            //         if !pred.test((v?).as_ref())? { return Ok(false) }
            //     }

            //     Ok(true)
            // },
            // &Self::Any(ref pred) => {
            //     // TODO: Have `IterableLike::any()` accept this `Predicate` type and use it instead of trait.
            //     for v in IterableLike::try_from(arg)? {
            //         if pred.test((v?).as_ref())? { return Ok(true) }
            //     }

            //     Ok(false)
            // },
            &Self::And(b) => Ok(bool::try_from(arg).map_err(|_| Error::NotBoolean)? && b),
            &Self::Or(b) => Ok(bool::try_from(arg).map_err(|_| Error::NotBoolean)? || b),
            &Self::Xor(b) => Ok(bool::try_from(arg).map_err(|_| Error::NotBoolean)? ^ b),
            &Self::Eq(ref n) => Ok(Number::try_from(arg).map_err(|_| Error::NotNumeric)?.val_cmp(&n) == Ordering::Equal),
            &Self::Ne(ref n) => Ok(Number::try_from(arg).map_err(|_| Error::NotNumeric)?.val_cmp(&n) != Ordering::Equal),
            &Self::Lt(ref n) => Ok(Number::try_from(arg).map_err(|_| Error::NotNumeric)?.val_cmp(&n) == Ordering::Less),
            &Self::Le(ref n) => Ok(Number::try_from(arg).map_err(|_| Error::NotNumeric)?.val_cmp(&n) != Ordering::Greater),
            &Self::Gt(ref n) => Ok(Number::try_from(arg).map_err(|_| Error::NotNumeric)?.val_cmp(&n) == Ordering::Greater),
            &Self::Ge(ref n) => Ok(Number::try_from(arg).map_err(|_| Error::NotNumeric)?.val_cmp(&n) != Ordering::Less),
            // &Self::HasKeyA(ref k) => {
            //     match arg {
            //         &MetaVal::Map(ref m) => Ok(m.contains_key(k)),
            //         _ => Err(Error::NotMapping),
            //     }
            // },
            _ => Ok(false),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Predicate2 {
    All,
    Any,
    And,
    Or,
    Xor,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    HasKey,
}
