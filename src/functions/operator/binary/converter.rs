use std::convert::TryInto;
use std::cmp::Ordering;

use itertools::Itertools;
use bigdecimal::BigDecimal;

use crate::metadata::types::MetaVal;
use crate::functions::Error;
use super::Predicate;

#[derive(Clone, Copy, Debug)]
pub enum Converter {
    Nth,
    StepBy,
    Chain,
    Zip,
    Map,
    Filter,
    SkipWhile,
    TakeWhile,
    Skip,
    Take,
    All,
    Any,
    Find,
    Position,
    Interleave,
    Intersperse,
    Chunks,
    Windows,

    Predicate(Predicate),
}

impl Converter {
    pub fn process<'mv>(&self, mv_a: MetaVal<'mv>, mv_b: MetaVal<'mv>) -> Result<MetaVal<'mv>, Error> {
        match self {
            _ => Ok(MetaVal::Nil)
        }
    }
}

#[cfg(test)]
mod tests {
}
