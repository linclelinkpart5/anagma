use std::convert::TryInto;
use std::cmp::Ordering;

use itertools::Itertools;
use bigdecimal::BigDecimal;

use crate::metadata::types::MetaVal;
use crate::functions::Error;
use super::Predicate;

#[derive(Clone, Copy, Debug)]
pub enum Converter {
}

impl Converter {
    pub fn process<'mv>(&self, mv: MetaVal<'mv>) -> Result<MetaVal<'mv>, Error> {
        match self {
            _ => Ok(MetaVal::Nil)
        }
    }
}

#[cfg(test)]
mod tests {
}
