use std::convert::TryInto;

use crate::metadata::types::MetaVal;
use crate::functions::Error;

#[derive(Clone, Copy, Debug)]
pub enum Predicate {
}

impl Predicate {
    pub fn process<'mv>(&self, mv: &'mv MetaVal<'mv>) -> Result<bool, Error> {
        match self {
            _ => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
}
