pub mod number_like;
pub mod stream_adaptor;

pub use self::number_like::NumberLike;
pub use self::stream_adaptor::StreamAdaptor;
pub use self::stream_adaptor::*;

use std::convert::TryInto;

use crate::metadata::types::MetaVal;
use crate::functions::Error;
use crate::functions::operator::UnaryPredicate;

#[derive(Clone, Copy)]
enum MinMax { Min, Max, }

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

    pub fn first(sa: StreamAdaptor) -> Result<MetaVal, Error> {
        sa.into_iter().next().ok_or(Error::EmptyStream)?
    }

    pub fn last(sa: StreamAdaptor) -> Result<MetaVal, Error> {
        let mut last = None;
        for res_mv in sa { last = Some(res_mv?); }
        last.ok_or(Error::EmptyStream)
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

    pub fn rev(sa: StreamAdaptor) -> Result<Vec<MetaVal>, Error> {
        let mut seq = Self::collect(sa)?;
        seq.reverse();
        Ok(seq)
    }

    pub fn sort(sa: StreamAdaptor) -> Result<Vec<MetaVal>, Error> {
        let mut seq = Self::collect(sa)?;
        // TODO: Use proper sort by key.
        seq.sort();
        Ok(seq)
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

    pub fn nth(sa: StreamAdaptor, n: usize) -> Result<MetaVal, Error> {
        let mut i = 0;
        for res_mv in sa {
            let mv = res_mv?;

            if i == n { return Ok(mv) }
            else { i += 1; }
        }

        Err(Error::OutOfBounds)
    }

    pub fn all(sa: StreamAdaptor, u_pred: UnaryPredicate) -> Result<bool, Error> {
        for res_mv in sa {
            let mv = res_mv?;
            if !u_pred.process(&mv)? { return Ok(false) }
        }

        Ok(true)
    }

    pub fn any(sa: StreamAdaptor, u_pred: UnaryPredicate) -> Result<bool, Error> {
        for res_mv in sa {
            let mv = res_mv?;
            if u_pred.process(&mv)? { return Ok(true) }
        }

        Ok(false)
    }

    // StepBy,
    // Chain,
    // Zip,
    // Map,
    // Filter,
    // SkipWhile,
    // TakeWhile,
    // Skip,
    // Take,
    // Find,
    // Position,
    // Interleave,
    // Intersperse,
    // Chunks,
    // Windows,
}
