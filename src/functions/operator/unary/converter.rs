use std::convert::TryInto;

use itertools::Itertools;

use crate::metadata::types::MetaVal;
use crate::functions::Error;
use crate::functions::operator::unary::Predicate;
use crate::functions::operator::unary::IterConsumer;

#[derive(Clone, Copy, Debug)]
pub enum Converter {
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
    Predicate(Predicate),
}

impl Converter {
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
                IterConsumer::MaxIn.process(seq.into_iter().map(Result::Ok))
            },
            &Self::MinIn => {
                let seq: Vec<_> = mv.try_into()?;
                IterConsumer::MinIn.process(seq.into_iter().map(Result::Ok))
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
                IterConsumer::Sum.process(seq.into_iter().map(Result::Ok))
            },
            &Self::Prod => {
                let seq: Vec<_> = mv.try_into()?;
                IterConsumer::Prod.process(seq.into_iter().map(Result::Ok))
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
