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
use crate::functions::util::value_producer::ValueProducer;
use crate::functions::util::value_producer::Fixed;
use crate::functions::util::value_producer::Flatten;
use crate::functions::util::value_producer::Dedup;
use crate::functions::util::value_producer::Unique;

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
        vp.into_iter().next().ok_or(Error::EmptyProducer)?
    }

    pub fn first_s(seq: Vec<MetaVal>) -> Result<MetaVal, Error> {
        seq.into_iter().next().ok_or(Error::EmptySequence)
    }

    pub fn last<'a, VP: ValueProducer<'a>>(vp: VP) -> Result<MetaVal<'a>, Error> {
        let mut last = None;
        for res_mv in vp { last = Some(res_mv?); }
        last.ok_or(Error::EmptyProducer)
    }

    pub fn last_s(seq: Vec<MetaVal>) -> Result<MetaVal, Error> {
        seq.into_iter().last().ok_or(Error::EmptySequence)
    }

    fn min_in_max_in<'a, VP: ValueProducer<'a>, EF: FnOnce() -> Error>(vp: VP, flag: MinMax, ef: EF) -> Result<NumberLike, Error> {
        let mut vp = vp.into_iter();
        match vp.next() {
            None => Err(ef()),
            Some(first_res_mv) => {
                let mut target_nl: NumberLike = first_res_mv?.try_into()?;

                for res_mv in vp {
                    let nl: NumberLike = res_mv?.try_into()?;
                    target_nl = match flag {
                        MinMax::Min => target_nl.val_min(nl),
                        MinMax::Max => target_nl.val_max(nl),
                    };
                }

                Ok(target_nl)
            }
        }
    }

    pub fn min_in<'a, VP: ValueProducer<'a>>(vp: VP) -> Result<NumberLike, Error> {
        Self::min_in_max_in(vp, MinMax::Min, || Error::EmptyProducer)
    }

    pub fn min_in_s(seq: Vec<MetaVal>) -> Result<NumberLike, Error> {
        Self::min_in_max_in(Fixed::new(seq), MinMax::Min, || Error::EmptySequence)
    }

    pub fn max_in<'a, VP: ValueProducer<'a>>(vp: VP) -> Result<NumberLike, Error> {
        Self::min_in_max_in(vp, MinMax::Max, || Error::EmptyProducer)
    }

    pub fn max_in_s(seq: Vec<MetaVal>) -> Result<NumberLike, Error> {
        Self::min_in_max_in(Fixed::new(seq), MinMax::Max, || Error::EmptySequence)
    }

    fn smart_sort_by<'mv>(a: &MetaVal<'mv>, b: &MetaVal<'mv>) -> std::cmp::Ordering {
        // Smooth over comparsions between integers and decimals.
        match (a, b) {
            (&MetaVal::Int(ref i), &MetaVal::Dec(ref d)) => {
                let i_d = (*i).into();
                // NOTE: Do this to avoid having to import other modules just for type inference.
                d.cmp(&i_d).reverse()
            },
            (&MetaVal::Dec(ref d), &MetaVal::Int(ref i)) => {
                let i_d = (*i).into();
                d.cmp(&i_d)
            },
            (na, nb) => na.cmp(&nb),
        }
    }

    fn rev_sort(mut seq: Vec<MetaVal>, flag: RevSort) -> Vec<MetaVal> {
        match flag {
            RevSort::Rev => seq.reverse(),
            RevSort::Sort => seq.sort_by(Self::smart_sort_by),
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
        match Self::flatten(Fixed::new(seq)).collect::<Result<Vec<_>, _>>() {
            Err(_) => unreachable!(),
            Ok(seq) => seq,
        }
    }

    pub fn dedup<'a, VP: ValueProducer<'a>>(vp: VP) -> Dedup<'a, VP> {
        Dedup::new(vp)
    }

    pub fn dedup_s(seq: Vec<MetaVal>) -> Vec<MetaVal> {
        match Self::dedup(Fixed::new(seq)).collect::<Result<Vec<_>, _>>() {
            Err(_) => unreachable!(),
            Ok(seq) => seq,
        }
    }

    pub fn unique<'a, VP: ValueProducer<'a>>(vp: VP) -> Unique<'a, VP> {
        Unique::new(vp)
    }

    pub fn unique_s(seq: Vec<MetaVal>) -> Vec<MetaVal> {
        match Self::unique(Fixed::new(seq)).collect::<Result<Vec<_>, _>>() {
            Err(_) => unreachable!(),
            Ok(seq) => seq,
        }
    }

    pub fn neg(number: NumberLike) -> NumberLike {
        match number {
            NumberLike::Integer(i) => NumberLike::Integer(-i),
            NumberLike::Decimal(d) => NumberLike::Decimal(-d),
        }
    }

    pub fn abs(number: NumberLike) -> NumberLike {
        match number {
            NumberLike::Integer(i) => NumberLike::Integer(i.abs()),
            NumberLike::Decimal(d) => NumberLike::Decimal(d.abs()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Impl;

    use crate::test_util::TestUtil as TU;

    use crate::metadata::types::MetaVal;
    use crate::functions::Error;
    use crate::functions::ErrorKind;
    use crate::functions::util::value_producer::Raw;
    use crate::functions::util::NumberLike;

    #[test]
    fn test_collect() {
        let inputs_and_expected = vec![
            (
                vec![],
                Ok(vec![]),
            ),
            (
                TU::core_nested_sequence().into_iter().map(Result::Ok).collect(),
                Ok(TU::core_nested_sequence()),
            ),
            (
                vec![Err(Error::Sentinel)],
                Err(ErrorKind::Sentinel),
            ),
            (
                vec![Ok(MetaVal::Bul(true)), Ok(MetaVal::Bul(true)), Err(Error::Sentinel)],
                Err(ErrorKind::Sentinel),
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = Impl::collect(Raw::new(input)).map_err(Into::<ErrorKind>::into);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_count() {
        let inputs_and_expected = vec![
            (
                vec![],
                Ok(0),
            ),
            (
                TU::core_nested_sequence().into_iter().map(Result::Ok).collect(),
                Ok(7),
            ),
            (
                vec![Err(Error::Sentinel)],
                Err(ErrorKind::Sentinel),
            ),
            (
                vec![Ok(MetaVal::Bul(true)), Ok(MetaVal::Bul(true)), Err(Error::Sentinel)],
                Err(ErrorKind::Sentinel),
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = Impl::count(Raw::new(input)).map_err(Into::<ErrorKind>::into);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_count_s() {
        let inputs_and_expected = vec![
            (vec![], 0usize),
            (vec![MetaVal::Bul(true)], 1),
            (TU::core_flat_sequence(), 5),
            (TU::core_nested_sequence(), 7),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = Impl::count_s(input);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_first() {
        let inputs_and_expected = vec![
            (
                vec![],
                Err(ErrorKind::EmptyProducer),
            ),
            (
                TU::core_nested_sequence().into_iter().map(Result::Ok).collect(),
                Ok(TU::core_nested_sequence()[0].clone()),
            ),
            (
                vec![Err(Error::Sentinel), Ok(MetaVal::Bul(true)), Ok(MetaVal::Bul(false))],
                Err(ErrorKind::Sentinel),
            ),
            (
                vec![Ok(MetaVal::Bul(true)), Ok(MetaVal::Bul(false)), Err(Error::Sentinel)],
                Ok(MetaVal::Bul(true)),
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = Impl::first(Raw::new(input)).map_err(Into::<ErrorKind>::into);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_first_s() {
        let inputs_and_expected = vec![
            (
                vec![],
                Err(ErrorKind::EmptySequence),
            ),
            (
                TU::core_nested_sequence(),
                Ok(TU::core_nested_sequence()[0].clone()),
            ),
            (
                vec![MetaVal::Bul(true), MetaVal::Bul(false)],
                Ok(MetaVal::Bul(true)),
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = Impl::first_s(input).map_err(Into::<ErrorKind>::into);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_last() {
        let inputs_and_expected = vec![
            (
                vec![],
                Err(ErrorKind::EmptyProducer),
            ),
            (
                TU::core_nested_sequence().into_iter().map(Result::Ok).collect(),
                Ok(TU::core_nested_sequence().pop().unwrap()),
            ),
            (
                vec![Ok(MetaVal::Bul(true)), Ok(MetaVal::Bul(false))],
                Ok(MetaVal::Bul(false)),
            ),
            (
                vec![Err(Error::Sentinel), Ok(MetaVal::Bul(true)), Ok(MetaVal::Bul(false))],
                Err(ErrorKind::Sentinel),
            ),
            (
                vec![Ok(MetaVal::Bul(true)), Ok(MetaVal::Bul(false)), Err(Error::Sentinel)],
                Err(ErrorKind::Sentinel),
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = Impl::last(Raw::new(input)).map_err(Into::<ErrorKind>::into);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_last_s() {
        let inputs_and_expected = vec![
            (
                vec![],
                Err(ErrorKind::EmptySequence),
            ),
            (
                TU::core_nested_sequence(),
                Ok(TU::core_nested_sequence().pop().unwrap()),
            ),
            (
                vec![MetaVal::Bul(true), MetaVal::Bul(false)],
                Ok(MetaVal::Bul(false)),
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = Impl::last_s(input).map_err(Into::<ErrorKind>::into);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_min_in() {
        let inputs_and_expected = vec![
            (
                vec![],
                Err(ErrorKind::EmptyProducer),
            ),
            (
                TU::core_number_sequence(2, false, true, false).into_iter().map(Result::Ok).collect(),
                Ok(NumberLike::Integer(-2)),
            ),
            (
                TU::core_number_sequence(2, true, true, false).into_iter().map(Result::Ok).collect(),
                Ok(NumberLike::Decimal(TU::d_raw(-25, 1))),
            ),
            (
                vec![Ok(TU::i(1))],
                Ok(NumberLike::Integer(1)),
            ),
            (
                vec![Ok(TU::i(1)), Ok(MetaVal::Bul(false))],
                Err(ErrorKind::NotNumeric),
            ),
            (
                vec![Ok(TU::i(1)), Err(Error::Sentinel)],
                Err(ErrorKind::Sentinel),
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = Impl::min_in(Raw::new(input)).map_err(Into::<ErrorKind>::into);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_min_in_s() {
        let inputs_and_expected = vec![
            (
                vec![],
                Err(ErrorKind::EmptySequence),
            ),
            (
                TU::core_number_sequence(2, false, true, false),
                Ok(NumberLike::Integer(-2)),
            ),
            (
                TU::core_number_sequence(2, true, true, false),
                Ok(NumberLike::Decimal(TU::d_raw(-25, 1))),
            ),
            (
                vec![TU::i(1)],
                Ok(NumberLike::Integer(1)),
            ),
            (
                vec![TU::i(1), MetaVal::Bul(true)],
                Err(ErrorKind::NotNumeric),
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = Impl::min_in_s(input).map_err(Into::<ErrorKind>::into);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_max_in() {
        let inputs_and_expected = vec![
            (
                vec![],
                Err(ErrorKind::EmptyProducer),
            ),
            (
                TU::core_number_sequence(2, false, true, false).into_iter().map(Result::Ok).collect(),
                Ok(NumberLike::Integer(2)),
            ),
            (
                TU::core_number_sequence(2, true, true, false).into_iter().map(Result::Ok).collect(),
                Ok(NumberLike::Decimal(TU::d_raw(25, 1))),
            ),
            (
                vec![Ok(TU::i(1))],
                Ok(NumberLike::Integer(1)),
            ),
            (
                vec![Ok(TU::i(1)), Ok(MetaVal::Bul(false))],
                Err(ErrorKind::NotNumeric),
            ),
            (
                vec![Ok(TU::i(1)), Err(Error::Sentinel)],
                Err(ErrorKind::Sentinel),
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = Impl::max_in(Raw::new(input)).map_err(Into::<ErrorKind>::into);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_max_in_s() {
        let inputs_and_expected = vec![
            (
                vec![],
                Err(ErrorKind::EmptySequence),
            ),
            (
                TU::core_number_sequence(2, false, true, false),
                Ok(NumberLike::Integer(2)),
            ),
            (
                TU::core_number_sequence(2, true, true, false),
                Ok(NumberLike::Decimal(TU::d_raw(25, 1))),
            ),
            (
                vec![TU::i(1)],
                Ok(NumberLike::Integer(1)),
            ),
            (
                vec![TU::i(1), MetaVal::Bul(true)],
                Err(ErrorKind::NotNumeric),
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = Impl::max_in_s(input).map_err(Into::<ErrorKind>::into);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_rev() {
        let inputs_and_expected = vec![
            (
                vec![],
                Ok(vec![]),
            ),
            (
                TU::core_nested_sequence().into_iter().map(Result::Ok).collect(),
                Ok({ let mut s = TU::core_nested_sequence(); s.reverse(); s }),
            ),
            (
                vec![Ok(TU::i(1))],
                Ok(vec![TU::i(1)]),
            ),
            (
                vec![Ok(TU::i(1)), Err(Error::Sentinel)],
                Err(ErrorKind::Sentinel),
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = Impl::rev(Raw::new(input)).map_err(Into::<ErrorKind>::into);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_rev_s() {
        let inputs_and_expected = vec![
            (
                vec![],
                vec![],
            ),
            (
                TU::core_nested_sequence(),
                { let mut s = TU::core_nested_sequence(); s.reverse(); s },
            ),
            (
                vec![TU::i(1)],
                vec![TU::i(1)],
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = Impl::rev_s(input);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_sort() {
        let inputs_and_expected = vec![
            (
                vec![],
                Ok(vec![]),
            ),
            (
                TU::core_number_sequence(2, false, true, true).into_iter().map(Result::Ok).collect(),
                Ok(vec![TU::i(-2), TU::d(-15, 1), TU::i(-1), TU::d(-5, 1), TU::i(0), TU::d(5, 1), TU::i(1), TU::d(15, 1), TU::i(2)]),
            ),
            (
                vec![Ok(TU::i(1))],
                Ok(vec![TU::i(1)]),
            ),
            (
                vec![Ok(TU::i(1)), Err(Error::Sentinel)],
                Err(ErrorKind::Sentinel),
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = Impl::sort(Raw::new(input)).map_err(Into::<ErrorKind>::into);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_sort_s() {
        let inputs_and_expected = vec![
            (
                vec![],
                vec![],
            ),
            (
                TU::core_number_sequence(2, false, true, true),
                vec![TU::i(-2), TU::d(-15, 1), TU::i(-1), TU::d(-5, 1), TU::i(0), TU::d(5, 1), TU::i(1), TU::d(15, 1), TU::i(2)],
            ),
            (
                vec![TU::i(1)],
                vec![TU::i(1)],
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = Impl::sort_s(input);
            assert_eq!(expected, produced);
        }
    }
}
