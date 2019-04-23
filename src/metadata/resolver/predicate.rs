use crate::metadata::types::MetaVal;

#[derive(Debug, Clone, Copy)]
pub enum Error {
    NotSequence,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::NotSequence => write!(f, "not a sequence"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Self::NotSequence => None,
        }
    }
}

/// Unary operations that take a reference to a meta value, and return a bare boolean.
#[derive(Clone, Copy, Debug)]
pub enum Predicate {
    // (Sequence<V>) -> bool
    AllEqual,
}

impl Predicate {
    pub fn process<'mv>(&self, val: &MetaVal<'mv>) -> Result<bool, Error> {
        match self {
            &Self::AllEqual => {
                match val {
                    &MetaVal::Seq(ref seq) => {
                        let mut it = seq.into_iter();

                        match it.next() {
                            None => Ok(true),
                            Some(first_mv) => {
                                for mv in it {
                                    if mv != first_mv { return Ok(false); }
                                }

                                Ok(true)
                            },
                        }
                    },
                    _ => Err(Error::NotSequence),
                }
            }
        }
    }
}
