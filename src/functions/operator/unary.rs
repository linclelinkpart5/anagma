pub mod converter;
pub mod predicate;
pub mod iter_consumer;
pub mod iter_adaptor;

pub use self::converter::Converter;
pub use self::predicate::Predicate;
pub use self::iter_consumer::IterConsumer;
pub use self::iter_adaptor::IterAdaptor;

use std::convert::TryInto;

use crate::metadata::types::MetaVal;
use crate::functions::Error;
use crate::functions::util::StreamAdaptor;
use crate::functions::util::FlattenAdaptor;
use crate::functions::util::DedupAdaptor;
use crate::functions::util::UniqueAdaptor;
use crate::functions::util::NumberLike;

#[derive(Clone, Copy)]
enum MinMax { Min, Max, }

#[derive(Clone, Copy)]
enum RevSort { Rev, Sort, }

#[derive(Clone, Copy)]
enum SumProd { Sum, Prod, }

/// Namespace for all the implementation of various functions in this module.
pub struct Impl;

impl Impl {
    pub fn collect(sa: StreamAdaptor) -> Result<Vec<MetaVal>, Error> {
        Ok(sa.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn count(sa: StreamAdaptor) -> Result<usize, Error> {
        let mut c: usize = 0;
        for res_mv in sa { res_mv?; c += 1; }
        Ok(c)
    }

    pub fn count_s(seq: Vec<MetaVal>) -> usize {
        seq.len()
    }

    pub fn first(sa: StreamAdaptor) -> Result<MetaVal, Error> {
        sa.into_iter().next().ok_or(Error::EmptyStream)?
    }

    pub fn first_s(seq: Vec<MetaVal>) -> Option<MetaVal> {
        seq.into_iter().next()
    }

    pub fn last(sa: StreamAdaptor) -> Result<MetaVal, Error> {
        let mut last = None;
        for res_mv in sa { last = Some(res_mv?); }
        last.ok_or(Error::EmptyStream)
    }

    pub fn last_s(seq: Vec<MetaVal>) -> Option<MetaVal> {
        seq.into_iter().last()
    }

    fn min_max(sa: StreamAdaptor, flag: MinMax) -> Result<NumberLike, Error> {
        let mut sa = sa.into_iter();
        match sa.next() {
            None => Err(Error::EmptySequence),
            Some(first_res_mv) => {
                let mut target_nl: NumberLike = first_res_mv?.try_into()?;

                for res_mv in sa {
                    let nl: NumberLike = res_mv?.try_into()?;
                    target_nl = match flag {
                        MinMax::Min => target_nl.min(nl),
                        MinMax::Max => target_nl.max(nl),
                    };
                }

                Ok(target_nl)
            }
        }
    }

    pub fn min(sa: StreamAdaptor) -> Result<NumberLike, Error> {
        Self::min_max(sa, MinMax::Min)
    }

    pub fn max(sa: StreamAdaptor) -> Result<NumberLike, Error> {
        Self::min_max(sa, MinMax::Max)
    }

    fn rev_sort(sa: StreamAdaptor, flag: RevSort) -> Result<Vec<MetaVal>, Error> {
        let mut seq = Self::collect(sa)?;
        match flag {
            RevSort::Rev => seq.reverse(),
            // TODO: Use proper sort by key.
            RevSort::Sort => seq.sort(),
        };
        Ok(seq)
    }

    pub fn rev(sa: StreamAdaptor) -> Result<Vec<MetaVal>, Error> {
        Self::rev_sort(sa, RevSort::Rev)
    }

    pub fn sort(sa: StreamAdaptor) -> Result<Vec<MetaVal>, Error> {
        Self::rev_sort(sa, RevSort::Sort)
    }

    fn sum_prod(sa: StreamAdaptor, flag: SumProd) -> Result<NumberLike, Error> {
        let mut total = match flag {
            SumProd::Sum => NumberLike::Integer(0),
            SumProd::Prod => NumberLike::Integer(1),
        };

        for res_mv in sa {
            let nl: NumberLike = res_mv?.try_into()?;

            match flag {
                SumProd::Sum => { total += nl; },
                SumProd::Prod => { total *= nl; },
            };
        }

        Ok(total)
    }

    pub fn sum(sa: StreamAdaptor) -> Result<NumberLike, Error> {
        Self::sum_prod(sa, SumProd::Sum)
    }

    pub fn prod(sa: StreamAdaptor) -> Result<NumberLike, Error> {
        Self::sum_prod(sa, SumProd::Prod)
    }

    pub fn all_equal(sa: StreamAdaptor) -> Result<bool, Error> {
        let mut sa = sa.into_iter();
        match sa.next() {
            None => Ok(true),
            Some(res_first_mv) => {
                let first_mv = res_first_mv?;
                for res_mv in sa {
                    if res_mv? != first_mv { return Ok(false) }
                }

                Ok(true)
            },
        }
    }

    pub fn flatten(sa: StreamAdaptor) -> Result<FlattenAdaptor, Error> {
        Ok(FlattenAdaptor::new(sa))
    }

    pub fn dedup(sa: StreamAdaptor) -> Result<DedupAdaptor, Error> {
        Ok(DedupAdaptor::new(sa))
    }

    pub fn unique(sa: StreamAdaptor) -> Result<UniqueAdaptor, Error> {
        Ok(UniqueAdaptor::new(sa))
    }
}

