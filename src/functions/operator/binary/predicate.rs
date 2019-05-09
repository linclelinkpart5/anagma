use std::convert::TryInto;

use crate::metadata::types::MetaVal;
use crate::functions::Error;

#[derive(Clone, Copy, Debug)]
pub enum Predicate {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl Predicate {
    pub fn process<'mv>(&self, mv_a: &'mv MetaVal<'mv>, mv_b: &'mv MetaVal<'mv>) -> Result<bool, Error> {
        match self {
            &Self::Eq => Ok(mv_a == mv_b),
            &Self::Ne => Ok(mv_a != mv_b),
            &Self::Lt => Ok(mv_a < mv_b),
            &Self::Le => Ok(mv_a <= mv_b),
            &Self::Gt => Ok(mv_a > mv_b),
            &Self::Ge => Ok(mv_a >= mv_b),
        }
    }
}

#[cfg(test)]
mod tests {
}
