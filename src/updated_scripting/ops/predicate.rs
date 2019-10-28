
use std::convert::TryFrom;
use std::convert::TryInto;

use crate::metadata::types::MetaVal;
use crate::updated_scripting::Error;
use crate::updated_scripting::util::IterableLike;

#[derive(Clone, Debug)]
pub enum Predicate1 {
    AllEqual,
    IsEmpty,
    Not,
}

impl Predicate1 {
    pub fn test(&self, mv: &MetaVal) -> Result<bool, Error> {
        match self {
            &Self::AllEqual => IterableLike::try_from(mv)?.all_equal(),
            &Self::IsEmpty => IterableLike::try_from(mv)?.is_empty(),
            &Self::Not => Ok(!bool::try_from(mv).map_err(|_| Error::NotBoolean)?),
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
