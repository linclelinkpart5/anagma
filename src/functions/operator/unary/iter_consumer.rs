use std::convert::TryInto;

use crate::metadata::types::MetaVal;
use crate::functions::Error;
use crate::functions::util::NumberLike;

#[derive(Clone, Copy, Debug)]
pub enum IterConsumer {
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

impl IterConsumer {
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
