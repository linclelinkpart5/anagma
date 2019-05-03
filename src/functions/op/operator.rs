use std::borrow::Cow;
use std::convert::TryInto;
use std::convert::TryFrom;

use itertools::Itertools;

use crate::functions::op::operand::Operand;
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
    Predicate(UnaryPredicate),
}

impl UnaryConverter {
    pub fn process<'mv>(&self, mv: MetaVal<'mv>) -> Result<MetaVal<'mv>, Error> {
        match self {
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
            &Self::MaxIn => {
                let seq: Vec<_> = mv.try_into()?;
                UnaryIterConsumer::MaxIn.process(seq.into_iter().map(Result::Ok))
            },
            &Self::MinIn => {
                let seq: Vec<_> = mv.try_into()?;
                UnaryIterConsumer::MinIn.process(seq.into_iter().map(Result::Ok))
            }
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
            &Self::Sum => {
                let seq: Vec<_> = mv.try_into()?;
                UnaryIterConsumer::Sum.process(seq.into_iter().map(Result::Ok))
            },
            &Self::Prod => {
                let seq: Vec<_> = mv.try_into()?;
                UnaryIterConsumer::Prod.process(seq.into_iter().map(Result::Ok))
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

            // All predicates are implicitly converters as well.
            &Self::Predicate(pred) => pred.process(&mv).map(MetaVal::Bul),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum UnaryIterConsumer {
    Collect,
    Rev,
    Sort,
    Count,
    First,
    Last,
    MaxIn,
    MinIn,
    Sum,
    Prod,
    AllEqual,
}

impl UnaryIterConsumer {
    pub fn process<'mv>(&self, mut it: impl Iterator<Item = Result<MetaVal<'mv>, Error>>) -> Result<MetaVal<'mv>, Error> {
        match self {
            &Self::Collect | &Self::Rev | &Self::Sort => {
                let mut seq = it.collect::<Result<Vec<_>, _>>()?;

                match self {
                    &Self::Collect => {},
                    // This should delegate to the converter version.
                    &Self::Rev => { seq.reverse(); },
                    // This should delegate to the converter version.
                    &Self::Sort => { seq.sort(); },
                    _ => unreachable!(),
                };

                Ok(MetaVal::Seq(seq))
            },
            &Self::Count => {
                let mut c: usize = 0;

                for res_mv in it {
                    res_mv?;
                    c += 1;
                }

                Ok(MetaVal::Int(c as i64))
            },
            &Self::First => it.next().ok_or(Error::EmptyStream)?,
            &Self::Last => {
                // This is done in order to bail if an error is encounterd midway.
                let mut last = None;
                for res_mv in it { last = Some(res_mv?); }
                last.ok_or(Error::EmptyStream)
            },
            &Self::MaxIn | &Self::MinIn => {
                match it.next() {
                    None => Err(Error::EmptySequence),
                    Some(first_res_mv) => {
                        let mut target_nl: NumberLike = first_res_mv?.try_into()?;

                        for res_mv in it {
                            let nl: NumberLike = res_mv?.try_into()?;
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
            &Self::Sum | &Self::Prod => {
                let mut total = match self {
                    &Self::Sum => NumberLike::Integer(0),
                    &Self::Prod => NumberLike::Integer(1),
                    _ => unreachable!(),
                };

                for res_mv in it {
                    let nl: NumberLike = res_mv?.try_into()?;

                    match self {
                        &Self::Sum => { total += nl; },
                        &Self::Prod => { total *= nl; },
                        _ => unreachable!(),
                    };
                }

                Ok(total.into())
            },
            &Self::AllEqual => {
                match it.next() {
                    None => Ok(MetaVal::Bul(true)),
                    Some(res_first_mv) => {
                        let first_mv = res_first_mv?;
                        for res_mv in it {
                            if res_mv? != first_mv { return Ok(MetaVal::Bul(false)); }
                        }

                        Ok(MetaVal::Bul(true))
                    },
                }
            },
        }
    }
}

// All predicates are converters.
impl From<UnaryPredicate> for UnaryConverter {
    fn from(p: UnaryPredicate) -> Self {
        Self::Predicate(p)
    }
}

impl TryFrom<UnaryConverter> for UnaryPredicate {
    type Error = Error;

    fn try_from(p: UnaryConverter) -> Result<Self, Self::Error> {
        match p {
            UnaryConverter::Predicate(p) => Ok(p),
            _ => Err(Error::NotPredicate),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum UnaryIterAdaptor {
    Flatten,
    Dedup,
    Unique,
}

#[derive(Clone, Copy, Debug)]
pub enum UnaryOp {
    Converter(UnaryConverter),
    IterConsumer(UnaryIterConsumer),
}

impl UnaryOp {
    pub fn process<'o>(&self, operand: Operand<'o>) -> Result<Operand<'o>, Error> {
        Err(Error::InvalidOperand)
    }
}

impl From<UnaryPredicate> for UnaryOp {
    fn from(p: UnaryPredicate) -> Self {
        Self::Converter(UnaryConverter::Predicate(p))
    }
}

impl TryFrom<UnaryOp> for UnaryPredicate {
    type Error = Error;

    fn try_from(op: UnaryOp) -> Result<Self, Self::Error> {
        match op {
            UnaryOp::Converter(UnaryConverter::Predicate(p)) => Ok(p),
            _ => Err(Error::NotPredicate),
        }
    }
}

impl From<UnaryConverter> for UnaryOp {
    fn from(c: UnaryConverter) -> Self {
        Self::Converter(c)
    }
}

impl TryFrom<UnaryOp> for UnaryConverter {
    type Error = Error;

    fn try_from(op: UnaryOp) -> Result<Self, Self::Error> {
        match op {
            UnaryOp::Converter(c) => Ok(c),
            _ => Err(Error::NotConverter),
        }
    }
}
