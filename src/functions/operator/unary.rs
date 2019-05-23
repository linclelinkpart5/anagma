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
use crate::functions::util::NumberLike;
use crate::functions::util::ValueProducer;
use crate::functions::util::Fixed;
use crate::functions::util::Flatten;
use crate::functions::util::Dedup;
use crate::functions::util::Unique;

#[derive(Clone, Copy)]
enum MinMax { Min, Max, }

#[derive(Clone, Copy)]
enum RevSort { Rev, Sort, }

#[derive(Clone, Copy)]
enum SumProd { Sum, Prod, }

/// Namespace for all the implementation of various functions in this module.
pub struct Impl;

impl Impl {
    pub fn collect<'a, VP: ValueProducer<'a>>(vp: VP) -> Result<Vec<MetaVal<'a>>, Error> {
        Ok(vp.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn count<'a, VP: ValueProducer<'a>>(vp: VP) -> Result<usize, Error> {
        let mut c: usize = 0;
        for res_mv in vp { res_mv?; c += 1; }
        Ok(c)
    }

    pub fn count_s(seq: Vec<MetaVal>) -> usize {
        seq.len()
    }

    pub fn first<'a, VP: ValueProducer<'a>>(vp: VP) -> Result<MetaVal<'a>, Error> {
        vp.into_iter().next().ok_or(Error::EmptyStream)?
    }

    pub fn first_s(seq: Vec<MetaVal>) -> Option<MetaVal> {
        seq.into_iter().next()
    }

    pub fn last<'a, VP: ValueProducer<'a>>(vp: VP) -> Result<MetaVal<'a>, Error> {
        let mut last = None;
        for res_mv in vp { last = Some(res_mv?); }
        last.ok_or(Error::EmptyStream)
    }

    pub fn last_s(seq: Vec<MetaVal>) -> Option<MetaVal> {
        seq.into_iter().last()
    }

    fn min_max<'a, VP: ValueProducer<'a>>(vp: VP, flag: MinMax) -> Result<NumberLike, Error> {
        let mut vp = vp.into_iter();
        match vp.next() {
            None => Err(Error::EmptySequence),
            Some(first_res_mv) => {
                let mut target_nl: NumberLike = first_res_mv?.try_into()?;

                for res_mv in vp {
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

    pub fn min<'a, VP: ValueProducer<'a>>(vp: VP) -> Result<NumberLike, Error> {
        Self::min_max(vp, MinMax::Min)
    }

    pub fn min_s(seq: Vec<MetaVal>) -> Result<NumberLike, Error> {
        Self::min_max(Fixed::new(seq), MinMax::Min)
    }

    pub fn max<'a, VP: ValueProducer<'a>>(vp: VP) -> Result<NumberLike, Error> {
        Self::min_max(vp, MinMax::Max)
    }

    pub fn max_s(seq: Vec<MetaVal>) -> Result<NumberLike, Error> {
        Self::min_max(Fixed::new(seq), MinMax::Max)
    }

    fn rev_sort(mut seq: Vec<MetaVal>, flag: RevSort) -> Vec<MetaVal> {
        match flag {
            RevSort::Rev => seq.reverse(),
            // TODO: Use proper sort by key.
            RevSort::Sort => seq.sort(),
        };
        seq
    }

    pub fn rev<'a, VP: ValueProducer<'a>>(vp: VP) -> Result<Vec<MetaVal<'a>>, Error> {
        let seq = Self::collect(vp)?;
        Ok(Self::rev_sort(seq, RevSort::Rev))
    }

    pub fn rev_s(seq: Vec<MetaVal>) -> Vec<MetaVal> {
        Self::rev_sort(seq, RevSort::Rev)
    }

    pub fn sort<'a, VP: ValueProducer<'a>>(vp: VP) -> Result<Vec<MetaVal<'a>>, Error> {
        let seq = Self::collect(vp)?;
        Ok(Self::rev_sort(seq, RevSort::Sort))
    }

    pub fn sort_s(seq: Vec<MetaVal>) -> Vec<MetaVal> {
        Self::rev_sort(seq, RevSort::Sort)
    }

    fn sum_prod<'a, VP: ValueProducer<'a>>(vp: VP, flag: SumProd) -> Result<NumberLike, Error> {
        let mut total = match flag {
            SumProd::Sum => NumberLike::Integer(0),
            SumProd::Prod => NumberLike::Integer(1),
        };

        for res_mv in vp {
            let nl: NumberLike = res_mv?.try_into()?;

            match flag {
                SumProd::Sum => { total += nl; },
                SumProd::Prod => { total *= nl; },
            };
        }

        Ok(total)
    }

    pub fn sum<'a, VP: ValueProducer<'a>>(vp: VP) -> Result<NumberLike, Error> {
        Self::sum_prod(vp, SumProd::Sum)
    }

    pub fn sum_s(seq: Vec<MetaVal>) -> Result<NumberLike, Error> {
        Self::sum_prod(Fixed::new(seq), SumProd::Sum)
    }

    pub fn prod<'a, VP: ValueProducer<'a>>(vp: VP) -> Result<NumberLike, Error> {
        Self::sum_prod(vp, SumProd::Prod)
    }

    pub fn prod_s(seq: Vec<MetaVal>) -> Result<NumberLike, Error> {
        Self::sum_prod(Fixed::new(seq), SumProd::Prod)
    }

    pub fn all_equal<'a, VP: ValueProducer<'a>>(vp: VP) -> Result<bool, Error> {
        let mut vp = vp.into_iter();
        match vp.next() {
            None => Ok(true),
            Some(res_first_mv) => {
                let first_mv = res_first_mv?;
                for res_mv in vp {
                    if res_mv? != first_mv { return Ok(false) }
                }

                Ok(true)
            },
        }
    }

    pub fn all_equal_rs(ref_seq: &Vec<MetaVal>) -> bool {
        let mut ref_seq = ref_seq.into_iter();
        match ref_seq.next() {
            None => true,
            Some(first_mv) => {
                for mv in ref_seq {
                    if mv != first_mv { return false }
                }

                true
            },
        }
    }

    pub fn flatten<'a, VP: ValueProducer<'a>>(vp: VP) -> Flatten<'a, VP> {
        Flatten::new(vp)
    }

    pub fn flatten_s(seq: Vec<MetaVal>) -> Vec<MetaVal> {
        Self::flatten(Fixed::new(seq)).collect::<Result<Vec<_>, _>>().unwrap()
    }

    pub fn dedup<'a, VP: ValueProducer<'a>>(vp: VP) -> Dedup<'a, VP> {
        Dedup::new(vp)
    }

    pub fn dedup_s(seq: Vec<MetaVal>) -> Vec<MetaVal> {
        let mut seq = seq;
        seq.dedup();
        seq
    }

    pub fn unique<'a, VP: ValueProducer<'a>>(vp: VP) -> Unique<'a, VP> {
        Unique::new(vp)
    }

    pub fn unique_s(seq: Vec<MetaVal>) -> Vec<MetaVal> {
        Self::unique(Fixed::new(seq)).collect::<Result<Vec<_>, _>>().unwrap()
    }
}

