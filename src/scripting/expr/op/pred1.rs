/// Represents predicates, which take a single meta value by reference and return a boolean result.

use std::cmp::Ordering;
use std::convert::TryInto;

use crate::metadata::types::MetaVal;
use crate::scripting::Error;
use crate::scripting::util::number_like::NumberLike;
use crate::scripting::util::ref_iterable_like::RefIterableLike;

#[derive(Clone)]
pub enum Pred1 {
    AllEqual,
    Not,
    All(Box<Pred1>),
    Any(Box<Pred1>),
    And(bool),
    Or(bool),
    Xor(bool),
    Eq(NumberLike),
    Ne(NumberLike),
    Lt(NumberLike),
    Le(NumberLike),
    Gt(NumberLike),
    Ge(NumberLike),
    #[cfg(test)] Raw(fn(&MetaVal) -> Result<bool, Error>),
}

impl Pred1 {
    pub fn test(&self, mv: &MetaVal) -> Result<bool, Error> {
        match self {
            &Self::AllEqual => {
                match mv {
                    &MetaVal::Seq(ref s) => RefIterableLike::from(s).all_equal(),
                    _ => Err(Error::NotSequence),
                }
            },
            &Self::Not => {
                match mv {
                    &MetaVal::Bul(b) => Ok(!b),
                    _ => Err(Error::NotBoolean),
                }
            },
            &Self::All(ref p) => {
                match mv {
                    &MetaVal::Seq(ref s) => RefIterableLike::from(s).all(p),
                    _ => Err(Error::NotSequence)?,
                }
            },
            &Self::Any(ref p) => {
                match mv {
                    &MetaVal::Seq(ref s) => RefIterableLike::from(s).any(p),
                    _ => Err(Error::NotSequence)?,
                }
            },
            &Self::And(b_a) => {
                Ok(b_a && match mv {
                    &MetaVal::Bul(b_b) => b_b,
                    _ => Err(Error::NotBoolean)?,
                })
            },
            &Self::Or(b_a) => {
                Ok(b_a || match mv {
                    &MetaVal::Bul(b_b) => b_b,
                    _ => Err(Error::NotBoolean)?,
                })
            },
            &Self::Xor(b_a) => {
                Ok(b_a ^ match mv {
                    &MetaVal::Bul(b_b) => b_b,
                    _ => Err(Error::NotBoolean)?,
                })
            },
            &Self::Eq(ref num_a) => {
                let ord = num_a.val_cmp(&(mv.try_into()?));
                Ok(ord == Ordering::Equal)
            },
            &Self::Ne(ref num_a) => {
                let ord = num_a.val_cmp(&(mv.try_into()?));
                Ok(ord != Ordering::Equal)
            },
            &Self::Lt(ref num_a) => {
                let ord = num_a.val_cmp(&(mv.try_into()?));
                Ok(ord == Ordering::Less)
            },
            &Self::Le(ref num_a) => {
                let ord = num_a.val_cmp(&(mv.try_into()?));
                Ok(ord == Ordering::Less || ord == Ordering::Equal)
            },
            &Self::Gt(ref num_a) => {
                let ord = num_a.val_cmp(&(mv.try_into()?));
                Ok(ord == Ordering::Greater)
            },
            &Self::Ge(ref num_a) => {
                let ord = num_a.val_cmp(&(mv.try_into()?));
                Ok(ord == Ordering::Greater || ord == Ordering::Equal)
            },
            #[cfg(test)] &Self::Raw(ref raw_fn) => raw_fn(&mv),
        }
    }
}
