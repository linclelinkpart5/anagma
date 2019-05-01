use std::borrow::Cow;
use std::convert::TryInto;
use std::convert::TryFrom;

use itertools::Itertools;

use crate::functions::op::operand::Operand;
use crate::functions::util::IterableLike;
use crate::functions::util::NumberLike;
use crate::functions::util::StreamAdaptor;
use crate::functions::Error;
use crate::metadata::types::MetaVal;

impl<'mv> TryFrom<MetaVal<'mv>> for Vec<MetaVal<'mv>> {
    type Error = Error;

    fn try_from(mv: MetaVal<'mv>) -> Result<Self, Self::Error> {
        match mv {
            MetaVal::Seq(seq) => Ok(seq),
            _ => Err(Error::NotSequence),
        }
    }
}

impl<'mv> TryFrom<&'mv MetaVal<'mv>> for &'mv Vec<MetaVal<'mv>> {
    type Error = Error;

    fn try_from(mv: &'mv MetaVal<'mv>) -> Result<Self, Self::Error> {
        match mv {
            &MetaVal::Seq(ref seq) => Ok(seq),
            _ => Err(Error::NotSequence),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum UnaryPredicate {
    AllEqual,
}

impl UnaryPredicate {
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

#[derive(Clone, Copy, Debug)]
pub enum UnaryConverter {
    // All unary predicates can be treated as unary converters.
    Predicate(UnaryPredicate),

    Count,
    First,
    Last,
    MaxIn,
    MinIn,
    Rev,
    Sort,
    Sum,
    Prod,
    Flatten,
    Dedup,
    Unique,
}

impl UnaryConverter {
    pub fn process<'mv>(&self, mv: MetaVal<'mv>) -> Result<MetaVal<'mv>, Error> {
        match self {
            &Self::Predicate(pred) => pred.process(&mv).map(MetaVal::Bul),

            &Self::Count => {
                let ref_seq: &Vec<_> = (&mv).try_into()?;
                Ok(MetaVal::Int(ref_seq.len() as i64))
            },
            &Self::First => {
                let seq: Vec<_> = mv.try_into()?;
                seq.into_iter().next().ok_or(Error::EmptySequence)
            },
            &Self::Last => {
                let seq: Vec<_> = mv.try_into()?;
                seq.into_iter().last().ok_or(Error::EmptySequence)
            },
            &Self::MaxIn | &Self::MinIn => {
                let seq: Vec<_> = mv.try_into()?;

                let mut it = seq.into_iter();

                match it.next() {
                    None => Err(Error::EmptySequence),
                    Some(first_mv) => {
                        let mut target_nl: NumberLike = first_mv.try_into()?;

                        for mv in it {
                            let nl: NumberLike = mv.try_into()?;
                            target_nl = match self {
                                &Self::MaxIn => target_nl.max(nl),
                                &Self::MinIn => target_nl.min(nl),
                                _ => unreachable!(),
                            };
                        }

                        Ok(target_nl.into())
                    }
                }
            },
            &Self::Rev => {
                let mut seq: Vec<_> = mv.try_into()?;
                seq.reverse();
                Ok(MetaVal::Seq(seq))
            },
            &Self::Sort => {
                let mut seq: Vec<_> = mv.try_into()?;
                seq.sort();
                Ok(MetaVal::Seq(seq))
            },
            &Self::Sum | &Self::Prod => {
                let seq: Vec<_> = mv.try_into()?;

                let mut total = match self {
                    &Self::Sum => NumberLike::Integer(0),
                    &Self::Prod => NumberLike::Integer(1),
                    _ => unreachable!(),
                };

                for mv in seq {
                    let nl: NumberLike = mv.try_into()?;

                    match self {
                        &Self::Sum => { total += nl; },
                        &Self::Prod => { total *= nl; },
                        _ => unreachable!(),
                    };
                }

                Ok(total.into())
            },
            &Self::Flatten => {
                let seq: Vec<_> = mv.try_into()?;
                let mut flattened = vec![];

                for mv in seq {
                    match mv {
                        MetaVal::Seq(mut seq) => flattened.append(&mut seq),
                        mv => flattened.push(mv),
                    }
                }
                Ok(MetaVal::Seq(flattened))
            },
            &Self::Dedup => {
                let mut seq: Vec<_> = mv.try_into()?;
                // TODO: Figure out equality rules.
                seq.dedup();
                Ok(MetaVal::Seq(seq))
            },
            &Self::Unique => {
                let seq: Vec<_> = mv.try_into()?;
                // TODO: Figure out equality rules.
                Ok(MetaVal::Seq(seq.into_iter().unique().collect()))
            },
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum UnaryStreamConsumer {
    Collect,
    Count,
    First,
    Last,
    MaxIn,
    MinIn,
    Rev,
    Sort,
    Sum,
    Prod,
    AllEqual,
}

impl UnaryStreamConsumer {
    pub fn process<'sa>(&self, mut sa: StreamAdaptor<'sa>) -> Result<MetaVal<'sa>, Error> {
        match self {
            &Self::Collect => sa.collect::<Result<Vec<_>, _>>().map(MetaVal::Seq),
            &Self::Count => {
                let mut c: usize = 0;

                for res_mv in sa {
                    res_mv?;
                    c += 1;
                }

                Ok(MetaVal::Int(c as i64))
            },
            &Self::First => sa.next().ok_or(Error::EmptyStream)?,
            &Self::Last => {
                // This is done in order to bail if an error is encounterd midway.
                let mut last = None;
                for res_mv in sa { last = Some(res_mv?); }
                last.ok_or(Error::EmptyStream)
            }
            _ => Ok(MetaVal::Nil),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum UnaryOp {
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

impl UnaryOp {
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

impl From<UnaryPredicate> for UnaryOp {
    fn from(up: UnaryPredicate) -> Self {
        match up {
            UnaryPredicate::AllEqual => Self::AllEqual,
        }
    }
}

impl TryFrom<UnaryOp> for UnaryPredicate {
    type Error = Error;

    fn try_from(uo: UnaryOp) -> Result<Self, Self::Error> {
        match uo {
            UnaryOp::AllEqual => Ok(Self::AllEqual),
            _ => Err(Error::NotPredicate),
        }
    }
}
