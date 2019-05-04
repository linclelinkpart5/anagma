use std::convert::TryInto;

use crate::metadata::types::MetaVal;
use crate::functions::Error;

#[derive(Clone, Copy, Debug)]
pub enum Predicate {
    AllEqual,
}

impl Predicate {
    pub fn process<'mv>(&self, mv: &'mv MetaVal<'mv>) -> Result<bool, Error> {
        match self {
            &Self::AllEqual => {
                let ref_seq: &Vec<_> = mv.try_into()?;

                let mut it = ref_seq.into_iter();

                match it.next() {
                    None => Ok(true),
                    Some(first_mv) => {
                        for mv in it {
                            if mv != first_mv { return Ok(false); }
                        }

                        Ok(true)
                    },
                }
            }
        }
    }
}
