use std::borrow::Cow;
use std::convert::TryInto;

use crate::functions::op::operand::Operand;
use crate::functions::util::iterable_like::IterableLike;
use crate::functions::util::number_like::NumberLike;
use crate::functions::Error;
use crate::metadata::types::MetaVal;

#[derive(Clone, Copy, Debug)]
pub enum Predicate {
    AllEqual,
}

pub fn all_equal_agnostic<'mv>(mut it: impl Iterator<Item = Result<Cow<'mv, MetaVal<'mv>>, &'static str>>) -> Result<bool, &'static str> {
    match it.next() {
        None => Ok(true),
        Some(res_first_mv) => {
            let first_mv = res_first_mv?;

            for res_mv in it {
                let mv = res_mv?;
                if mv != first_mv {
                    return Ok(false);
                }
            }

            Ok(true)
        },
    }
}

impl Predicate {
    pub fn process<'mv>(&self, mv: &'mv MetaVal<'mv>) -> Result<bool, Error> {
        match self {
            &Self::AllEqual => {
                match mv {
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
                    Operand::StreamAdaptor(sa) => sa.collect::<Result<Vec<_>, _>>().map(MetaVal::Seq).map(Cow::Owned).map(Operand::Value),
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
                    Operand::StreamAdaptor(sa) => {
                        let mut c: usize = 0;

                        for res_mv in sa {
                            res_mv?;
                            c += 1;
                        }

                        Ok(Operand::Value(Cow::Owned(MetaVal::Int(c as i64))))
                    },
                    _ => Err(Error::InvalidOperand),
                }
            },
            &Self::First => {
                match operand {
                    Operand::Value(mv) => {
                        match mv.into_owned() {
                            MetaVal::Seq(seq) => {
                                seq.into_iter().next().map(Cow::Owned).map(Operand::Value).ok_or(Error::EmptySequence)
                            },
                            _ => Err(Error::NotSequence),
                        }
                    },
                    Operand::StreamAdaptor(mut sa) => {
                        sa.next().ok_or(Error::EmptyStream)?.map(Cow::Owned).map(Operand::Value)
                    },
                    _ => Err(Error::InvalidOperand),
                }
            },
            &Self::Last => {
                match operand {
                    Operand::Value(mv) => {
                        match mv.into_owned() {
                            MetaVal::Seq(seq) => {
                                seq.into_iter().last().map(Cow::Owned).map(Operand::Value).ok_or(Error::EmptySequence)
                            },
                            _ => Err(Error::NotSequence),
                        }
                    },
                    Operand::StreamAdaptor(sa) => {
                        let mut last = None;
                        for res_mv in sa {
                            last = Some(res_mv?);
                        }
                        last.ok_or(Error::EmptyStream).map(Cow::Owned).map(Operand::Value)
                    },
                    _ => Err(Error::InvalidOperand),
                }
            },
            &Self::MaxIn | &Self::MinIn => {
                let mut m: Option<NumberLike> = None;

                let il: IterableLike<'_> = operand.try_into()?;

                for mv in il {
                    let num: NumberLike = mv?.try_into()?;

                    m = Some(
                        match m {
                            None => num,
                            Some(curr_m) => {
                                match self {
                                    &Self::MaxIn => curr_m.max(num),
                                    &Self::MinIn => curr_m.min(num),
                                    _ => unreachable!(),
                                }
                            },
                        }
                    );
                }

                Ok(Operand::Value(Cow::Owned(m.ok_or(Error::EmptyIterable)?.into())))
            },
            // Not finished yet!
            _ => Ok(Operand::Value(Cow::Owned(MetaVal::Nil))),
        }
    }
}
