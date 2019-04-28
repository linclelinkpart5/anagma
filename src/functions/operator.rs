use std::borrow::Cow;

use crate::functions::operand::Operand;
use crate::metadata::types::MetaVal;

#[derive(Debug, Copy, Clone)]
pub enum Error {
    NotIterable,
    NotSequence,
    InvalidOperand,
    EmptySequence,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::NotIterable => write!(f, "not an iterable"),
            Self::NotSequence => write!(f, "not a sequence"),
            Self::InvalidOperand => write!(f, "invalid operand"),
            Self::EmptySequence => write!(f, "empty sequence"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Self::NotIterable => None,
            Self::NotSequence => None,
            Self::InvalidOperand => None,
            Self::EmptySequence => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Unary {
    // (Iterable<V>) -> Sequence<V>
    Collect,
    // (Iterable<V>) -> Integer
    Count,
    // (Iterable<V>) -> V
    First,
    // (Iterable<V>) -> V
    Last,
    // (Iterable<Number>) -> Number
    MaxIn,
    // (Iterable<Number>) -> Number
    MinIn,
    // (Iterable<V>) -> Sequence<V>
    Rev,
    // (Iterable<Number>) -> Number
    Sum,
    // (Iterable<Number>) -> Number
    Product,
    // (Iterable<V>) -> Boolean
    AllEqual,
    // (Iterable<V>) -> Sequence<V>
    Sort,

    // (Sequence<V>) -> Sequence<V>
    // (Stream<V>) -> Stream<V>
    Flatten,
    // (Sequence<V>) -> Sequence<V>
    // (Stream<V>) -> Stream<V>
    Dedup,
    // (Sequence<V>) -> Sequence<V>
    // (Stream<V>) -> Stream<V>
    Unique,
}

fn operand_as_seq<'o>(operand: Operand<'o>) -> Result<Vec<MetaVal<'o>>, Error> {
    match operand {
        Operand::Value(mv) => {
            match mv.into_owned() {
                MetaVal::Seq(seq) => Ok(seq),
                _ => Err(Error::NotSequence),
            }
        },
        _ => Err(Error::InvalidOperand),
    }
}

fn operand_as_seq_ref<'o>(operand: &'o Operand<'o>) -> Result<&'o Vec<MetaVal<'o>>, Error> {
    match operand {
        Operand::Value(ref mv) => {
            match mv.as_ref() {
                &MetaVal::Seq(ref seq) => Ok(seq),
                _ => Err(Error::NotSequence),
            }
        },
        _ => Err(Error::InvalidOperand),
    }
}

impl Unary {
    pub fn process<'o>(&self, operand: Operand<'o>) -> Result<Operand<'o>, Error> {
        match self {
            &Self::Collect => {
                match operand {
                    Operand::Value(mv) => {
                        match mv.into_owned() {
                            MetaVal::Seq(seq) => Ok(Operand::Value(Cow::Owned(MetaVal::Seq(seq)))),
                            _ => Err(Error::NotSequence),
                        }
                    },
                    _ => Err(Error::InvalidOperand),
                }
            },
            &Self::Count => {
                // Only a reference is needed here for sequences.
                match operand {
                    Operand::Value(ref mv) => {
                        match mv.as_ref() {
                            &MetaVal::Seq(ref seq) => Ok(Operand::Value(Cow::Owned(MetaVal::Int(seq.len() as i64)))),
                            _ => Err(Error::NotSequence),
                        }
                    },
                    _ => Err(Error::InvalidOperand),
                }
            },
            &Self::First => {
                match operand {
                    Operand::Value(mv) => {
                        match mv.into_owned() {
                            MetaVal::Seq(seq) => {
                                let mut it = seq.into_iter();
                                it.next().map(Cow::Owned).map(Operand::Value).ok_or(Error::EmptySequence)
                            },
                            _ => Err(Error::NotSequence),
                        }
                    },
                    _ => Err(Error::InvalidOperand),
                }
            },
            &Self::Last => {
                match operand {
                    Operand::Value(mv) => {
                        match mv.into_owned() {
                            MetaVal::Seq(seq) => {
                                let it = seq.into_iter();
                                it.last().map(Cow::Owned).map(Operand::Value).ok_or(Error::EmptySequence)
                            },
                            _ => Err(Error::NotSequence),
                        }
                    },
                    _ => Err(Error::InvalidOperand),
                }
            },
            // Not finished yet!
            _ => Ok(Operand::Value(Cow::Owned(MetaVal::Nil))),
        }
    }
}
