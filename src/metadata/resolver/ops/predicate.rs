use crate::metadata::types::MetaVal;

/// Unary operations that take a reference to a meta value, and return a bare boolean.
#[derive(Clone, Copy, Debug)]
pub enum Predicate {
    // (Sequence<V>) -> bool
    AllEqualS,
}

impl Predicate {
    pub fn process<'mv>(&self, val: &MetaVal<'mv>) -> Result<bool, &'static str> {
        match self {
            &Self::AllEqualS => {
                match val {
                    &MetaVal::Seq(ref seq) => {
                        let mut it = seq.into_iter();

                        match it.next() {
                            None => Ok(true),
                            Some(first_mv) => {
                                for mv in it {
                                    if mv != first_mv {
                                        return Ok(false);
                                    }
                                }

                                Ok(true)
                            },
                        }
                    },
                    _ => Err("not a sequence"),
                }
            }
        }
    }
}
