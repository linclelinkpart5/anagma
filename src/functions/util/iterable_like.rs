use std::borrow::Cow;
use std::convert::TryFrom;
use std::convert::TryInto;

use crate::metadata::types::MetaVal;
use crate::functions::Error;
use crate::functions::util::value_producer::ValueProducer;
use crate::functions::util::value_producer::Flatten;
use crate::functions::util::value_producer::Dedup;
use crate::functions::util::value_producer::Unique;
use crate::functions::util::value_producer::Filter;
use crate::functions::util::value_producer::Map;
use crate::functions::util::value_producer::StepBy;
use crate::functions::util::value_producer::Chain;
use crate::functions::util::value_producer::Zip;
use crate::functions::util::value_producer::Skip;
use crate::functions::util::value_producer::Take;
use crate::functions::util::value_producer::SkipWhile;
use crate::functions::util::value_producer::TakeWhile;
use crate::functions::operand::Operand;
use crate::functions::util::NumberLike;
use crate::functions::util::UnaryPred;
use crate::functions::util::UnaryConv;

#[derive(Clone, Copy)]
enum MinMax { Min, Max, }

#[derive(Clone, Copy)]
enum RevSort { Rev, Sort, }

#[derive(Clone, Copy)]
enum SumProd { Sum, Prod, }

#[derive(Clone, Copy)]
enum AllAny { All, Any, }

impl AllAny {
    fn target(self) -> bool {
        match self {
            Self::All => false,
            Self::Any => true,
        }
    }
}

/// Represents one of several different kinds of iterables, producing meta values.
pub enum IterableLike<'il> {
    Sequence(Vec<MetaVal<'il>>),
    Producer(ValueProducer<'il>),
}

impl<'il> IterableLike<'il> {
    pub fn is_lazy(&self) -> bool {
        match self {
            &Self::Sequence(..) => false,
            &Self::Producer(..) => true,
        }
    }

    pub fn is_eager(&self) -> bool {
        !self.is_lazy()
    }

    pub fn collect(self) -> Result<Vec<MetaVal<'il>>, Error> {
        match self {
            Self::Sequence(s) => Ok(s),
            Self::Producer(p) => p.collect::<Result<Vec<_>, _>>(),
        }
    }

    pub fn count(self) -> Result<usize, Error> {
        match self {
            Self::Sequence(s) => Ok(s.len()),
            Self::Producer(p) => {
                let mut c: usize = 0;
                for res_mv in p { res_mv?; c += 1; }
                Ok(c)
            },
        }
    }

    pub fn first(self) -> Result<MetaVal<'il>, Error> {
        match self {
            Self::Sequence(s) => s.into_iter().next().ok_or(Error::EmptySequence),
            Self::Producer(p) => p.into_iter().next().ok_or(Error::EmptyProducer)?,
        }
    }

    pub fn last(self) -> Result<MetaVal<'il>, Error> {
        match self {
            Self::Sequence(s) => s.into_iter().last().ok_or(Error::EmptySequence),
            Self::Producer(p) => {
                let mut last = None;
                for res_mv in p { last = Some(res_mv?); }
                last.ok_or(Error::EmptyProducer)
            },
        }
    }

    fn min_in_max_in(self, flag: MinMax) -> Result<NumberLike, Error> {
        let (new_p, err) = match self {
            Self::Sequence(s) => (ValueProducer::from(s), Error::EmptySequence),
            Self::Producer(p) => (p, Error::EmptyProducer),
        };

        let mut it = new_p.into_iter();
        match it.next() {
            None => Err(err),
            Some(first_res_mv) => {
                let mut target_nl: NumberLike = first_res_mv?.try_into()?;

                for res_mv in it {
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

    pub fn min_in(self) -> Result<NumberLike, Error> {
        self.min_in_max_in(MinMax::Min)
    }

    pub fn max_in(self) -> Result<NumberLike, Error> {
        self.min_in_max_in(MinMax::Max)
    }

    fn smart_sort_by<'mv>(a: &MetaVal<'mv>, b: &MetaVal<'mv>) -> std::cmp::Ordering {
        // Smooth over comparsions between integers and decimals.
        // TODO: Create a stable ordering for equal integers and decimals. (e.g. I(5) vs D(5.0))
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

    fn rev_sort(self, flag: RevSort) -> Result<Vec<MetaVal<'il>>, Error> {
        let mut new_s = self.collect()?;

        match flag {
            RevSort::Rev => new_s.reverse(),
            RevSort::Sort => new_s.sort_by(Self::smart_sort_by),
        };

        Ok(new_s)
    }

    pub fn rev(self) -> Result<Vec<MetaVal<'il>>, Error> {
        self.rev_sort(RevSort::Rev)
    }

    pub fn sort(self) -> Result<Vec<MetaVal<'il>>, Error> {
        self.rev_sort(RevSort::Sort)
    }

    fn sum_prod(self, flag: SumProd) -> Result<NumberLike, Error> {
        let mut total = match flag {
            SumProd::Sum => NumberLike::Integer(0),
            SumProd::Prod => NumberLike::Integer(1),
        };

        let new_p = match self {
            Self::Sequence(s) => ValueProducer::from(s),
            Self::Producer(p) => p,
        };

        for res_mv in new_p {
            let nl: NumberLike = res_mv?.try_into()?;

            match flag {
                SumProd::Sum => { total += nl; },
                SumProd::Prod => { total *= nl; },
            };
        }

        Ok(total)
    }

    pub fn sum(self) -> Result<NumberLike, Error> {
        self.sum_prod(SumProd::Sum)
    }

    pub fn prod(self) -> Result<NumberLike, Error> {
        self.sum_prod(SumProd::Prod)
    }

    fn all_equal_agnostic<'a, I>(it: I) -> Result<bool, Error>
    where
        I: Iterator<Item = Result<Cow<'a, MetaVal<'a>>, Error>>,
    {
        let mut it = it.into_iter();
        Ok(match it.next() {
            None => true,
            Some(res_first_mv) => {
                let first_mv = res_first_mv?;
                for res_mv in it {
                    let mv = res_mv?;
                    if mv != first_mv { return Ok(false) }
                }

                true
            },
        })
    }

    fn all_equal_s<'a>(r_s: &Vec<MetaVal<'a>>) -> Result<bool, Error> {
        Self::all_equal_agnostic(r_s.into_iter().map(Cow::Borrowed).map(Result::Ok))
    }

    fn all_equal_pred<'a>(ref_mv: &MetaVal<'a>) -> Result<bool, Error> {
        // Conforms to the predicate interface.
        match ref_mv {
            &MetaVal::Seq(ref s) => Self::all_equal_s(&s),
            _ => Err(Error::NotSequence),
        }
    }

    fn all_equal_p<'a>(p: ValueProducer<'a>) -> Result<bool, Error> {
        Self::all_equal_agnostic(p.map(|res| res.map(Cow::Owned)))
    }

    pub fn all_equal(self) -> Result<bool, Error> {
        match self {
            Self::Sequence(ref s) => Self::all_equal_s(&s),
            Self::Producer(p) => Self::all_equal_p(p),
        }
    }

    pub fn flatten(self) -> Result<Self, Error> {
        Ok(match self {
            Self::Sequence(s) => Self::Sequence(Flatten::new(s.into()).collect::<Result<Vec<_>, _>>()?),
            Self::Producer(p) => Self::Producer(ValueProducer::Flatten(Flatten::new(p))),
        })
    }

    pub fn dedup(self) -> Result<Self, Error> {
        Ok(match self {
            Self::Sequence(s) => Self::Sequence(Dedup::new(s.into()).collect::<Result<Vec<_>, _>>()?),
            Self::Producer(p) => Self::Producer(ValueProducer::Dedup(Dedup::new(p))),
        })
    }

    pub fn unique(self) -> Result<Self, Error> {
        Ok(match self {
            Self::Sequence(s) => Self::Sequence(Unique::new(s.into()).collect::<Result<Vec<_>, _>>()?),
            Self::Producer(p) => Self::Producer(ValueProducer::Unique(Unique::new(p))),
        })
    }

    pub fn nth(self, n: usize) -> Result<MetaVal<'il>, Error> {
        match self {
            Self::Sequence(s) => s.into_iter().nth(n).ok_or(Error::OutOfBounds),
            Self::Producer(p) => {
                let mut i = 0;
                for res_mv in p {
                    let mv = res_mv?;

                    if i == n { return Ok(mv) }
                    else { i += 1; }
                }

                Err(Error::OutOfBounds)
            },
        }
    }

    fn all_any(self, u_pred: UnaryPred, flag: AllAny) -> Result<bool, Error> {
        let new_p = match self {
            Self::Sequence(s) => ValueProducer::from(s),
            Self::Producer(p) => p,
        };

        let target = flag.target();
        for res_mv in new_p {
            let mv = res_mv?;
            if u_pred(&mv)? == target { return Ok(target) }
        }

        Ok(!target)
    }

    pub fn all(self, u_pred: UnaryPred) -> Result<bool, Error> {
        self.all_any(u_pred, AllAny::All)
    }

    pub fn any(self, u_pred: UnaryPred) -> Result<bool, Error> {
        self.all_any(u_pred, AllAny::Any)
    }

    pub fn find(self, u_pred: UnaryPred) -> Result<MetaVal<'il>, Error> {
        let new_p = match self {
            Self::Sequence(s) => ValueProducer::from(s),
            Self::Producer(p) => p,
        };

        for res_mv in new_p {
            let mv = res_mv?;
            if u_pred(&mv)? { return Ok(mv) }
        }

        Err(Error::ItemNotFound)
    }

    pub fn position(self, u_pred: UnaryPred) -> Result<usize, Error> {
        let new_p = match self {
            Self::Sequence(s) => ValueProducer::from(s),
            Self::Producer(p) => p,
        };

        let mut i = 0;
        for res_mv in new_p {
            let mv = res_mv?;
            if u_pred(&mv)? { return Ok(i) }
            i += 1;
        }

        Err(Error::ItemNotFound)
    }

    pub fn filter(self, u_pred: UnaryPred) -> Result<Self, Error> {
        Ok(match self {
            Self::Sequence(s) => Self::Sequence(Filter::new(s.into(), u_pred).collect::<Result<Vec<_>, _>>()?),
            Self::Producer(p) => Self::Producer(ValueProducer::Filter(Filter::new(p, u_pred))),
        })
    }

    pub fn map(self, u_conv: UnaryConv) -> Result<Self, Error> {
        Ok(match self {
            Self::Sequence(s) => Self::Sequence(Map::new(s.into(), u_conv).collect::<Result<Vec<_>, _>>()?),
            Self::Producer(p) => Self::Producer(ValueProducer::Map(Map::new(p, u_conv))),
        })
    }

    pub fn step_by(self, step: usize) -> Result<Self, Error> {
        Ok(match self {
            Self::Sequence(s) => Self::Sequence(StepBy::new(s.into(), step)?.collect::<Result<Vec<_>, _>>()?),
            Self::Producer(p) => Self::Producer(ValueProducer::StepBy(StepBy::new(p, step)?)),
        })
    }

    pub fn chain(self, other: IterableLike<'il>) -> Self {
        let (new_p_a, new_p_b) = match (self, other) {
            (Self::Sequence(s_a), Self::Sequence(s_b)) => {
                let mut s_a = s_a;
                s_a.extend(s_b);
                return Self::Sequence(s_a)
            },
            (Self::Sequence(s_a), Self::Producer(p_b)) => (ValueProducer::from(s_a), p_b),
            (Self::Producer(p_a), Self::Sequence(s_b)) => (p_a, ValueProducer::from(s_b)),
            (Self::Producer(p_a), Self::Producer(p_b)) => (p_a, p_b),
        };

        Self::Producer(ValueProducer::Chain(Chain::new(new_p_a, new_p_b)))
    }

    pub fn zip(self, other: IterableLike<'il>) -> Result<Self, Error> {
        let collect_after = self.is_eager() && other.is_eager();
        let (new_p_a, new_p_b) = match (self, other) {
            (Self::Sequence(s_a), Self::Sequence(s_b)) => (ValueProducer::from(s_a), ValueProducer::from(s_b)),
            (Self::Sequence(s_a), Self::Producer(p_b)) => (ValueProducer::from(s_a), p_b),
            (Self::Producer(p_a), Self::Sequence(s_b)) => (p_a, ValueProducer::from(s_b)),
            (Self::Producer(p_a), Self::Producer(p_b)) => (p_a, p_b),
        };

        let ret_p = ValueProducer::Zip(Zip::new(new_p_a, new_p_b));

        Ok(match collect_after {
            true => Self::Sequence(ret_p.try_into()?),
            false => Self::Producer(ret_p),
        })
    }

    pub fn skip(self, n: usize) -> Self {
        match self {
            Self::Sequence(s) => {
                let mut s = s;
                if n >= s.len() { Self::Sequence(vec![]) }
                else { Self::Sequence(s.split_off(n)) }
            },
            Self::Producer(s) => Self::Producer(ValueProducer::Skip(Skip::new(s, n))),
        }
    }

    pub fn take(self, n: usize) -> Self {
        match self {
            Self::Sequence(s) => {
                let mut s = s;
                s.truncate(n);
                Self::Sequence(s)
            },
            Self::Producer(s) => Self::Producer(ValueProducer::Take(Take::new(s, n))),
        }
    }

    pub fn skip_while(self, u_pred: UnaryPred) -> Result<Self, Error> {
        let collect_after = self.is_eager();
        let p = ValueProducer::from(self);

        let ret_p = ValueProducer::SkipWhile(SkipWhile::new(p, u_pred));

        Ok(match collect_after {
            true => Self::Sequence(ret_p.try_into()?),
            false => Self::Producer(ret_p),
        })
    }

    pub fn take_while(self, u_pred: UnaryPred) -> Result<Self, Error> {
        let collect_after = self.is_eager();
        let p = ValueProducer::from(self);

        let ret_p = ValueProducer::TakeWhile(TakeWhile::new(p, u_pred));

        Ok(match collect_after {
            true => Self::Sequence(ret_p.try_into()?),
            false => Self::Producer(ret_p),
        })
    }

    // pub fn intersperse(self, mv: MetaVal<'il>) -> Self {
    // }

    // pub fn interleave(self, other: IterableLike<'il>) -> Self {
    // }
}

impl<'il> From<IterableLike<'il>> for Operand<'il> {
    fn from(il: IterableLike<'il>) -> Self {
        match il {
            IterableLike::Sequence(sequence) => Self::Value(MetaVal::Seq(sequence)),
            IterableLike::Producer(producer) => Self::Producer(producer),
        }
    }
}

impl<'il> TryFrom<Operand<'il>> for IterableLike<'il> {
    type Error = Error;

    fn try_from(value: Operand<'il>) -> Result<Self, Self::Error> {
        match value {
            Operand::Value(mv) => Self::try_from(mv),
            Operand::Producer(s) => Ok(Self::Producer(s)),
            _ => Err(Error::NotIterable),
        }
    }
}

impl<'il> TryFrom<MetaVal<'il>> for IterableLike<'il> {
    type Error = Error;

    fn try_from(value: MetaVal<'il>) -> Result<Self, Self::Error> {
        match value {
            MetaVal::Seq(s) => Ok(Self::Sequence(s)),
            _ => Err(Error::NotIterable),
        }
    }
}

impl<'il> From<IterableLike<'il>> for ValueProducer<'il> {
    fn from(il: IterableLike<'il>) -> Self {
        match il {
            IterableLike::Sequence(s) => s.into(),
            IterableLike::Producer(p) => p,
        }
    }
}

impl<'il> TryFrom<IterableLike<'il>> for Vec<MetaVal<'il>> {
    type Error = Error;

    fn try_from(il: IterableLike<'il>) -> Result<Self, Self::Error> {
        match il {
            IterableLike::Sequence(s) => Ok(s),
            IterableLike::Producer(p) => p.collect(),
        }
    }
}

impl<'il> From<Vec<MetaVal<'il>>> for IterableLike<'il> {
    fn from(s: Vec<MetaVal<'il>>) -> Self {
        IterableLike::Sequence(s)
    }
}

impl<'il> From<ValueProducer<'il>> for IterableLike<'il> {
    fn from(p: ValueProducer<'il>) -> Self {
        IterableLike::Producer(p)
    }
}

impl<'il> IntoIterator for IterableLike<'il> {
    type Item = Result<MetaVal<'il>, Error>;
    type IntoIter = IteratorLike<'il>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Sequence(s) => IteratorLike::Sequence(s.into_iter()),
            Self::Producer(s) => IteratorLike::Producer(s),
        }
    }
}

pub enum IteratorLike<'il> {
    Sequence(std::vec::IntoIter<MetaVal<'il>>),
    Producer(ValueProducer<'il>),
}

impl<'il> Iterator for IteratorLike<'il> {
    type Item = Result<MetaVal<'il>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            &mut Self::Sequence(ref mut it) => it.next().map(Result::Ok),
            &mut Self::Producer(ref mut it) => it.next(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::IterableLike as IL;

    use crate::test_util::TestUtil as TU;

    use crate::metadata::types::MetaVal;
    use crate::functions::Error;
    use crate::functions::ErrorKind;
    use crate::functions::util::value_producer::ValueProducer as VP;
    use crate::functions::util::NumberLike;

    #[test]
    fn test_collect() {
        let inputs_and_expected: Vec<(IL, Result<Vec<MetaVal>, ErrorKind>)> = vec![
            (
                vec![].into(),
                Ok(vec![]),
            ),
            (
                TU::core_nested_sequence().into(),
                Ok(TU::core_nested_sequence()),
            ),
            (
                VP::fixed(vec![]).into(),
                Ok(vec![]),
            ),
            (
                VP::fixed(TU::core_nested_sequence()).into(),
                Ok(TU::core_nested_sequence()),
            ),
            (
                VP::raw(vec![Err(Error::Sentinel)]).into(),
                Err(ErrorKind::Sentinel),
            ),
            (
                VP::raw(vec![Ok(MetaVal::Bul(true)), Ok(MetaVal::Bul(true)), Err(Error::Sentinel)]).into(),
                Err(ErrorKind::Sentinel),
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = input.collect().map_err(ErrorKind::from);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_count() {
        let inputs_and_expected: Vec<(IL, Result<usize, ErrorKind>)> = vec![
            (
                vec![].into(),
                Ok(0),
            ),
            (
                TU::core_nested_sequence().into(),
                Ok(7),
            ),
            (
                VP::fixed(vec![]).into(),
                Ok(0),
            ),
            (
                VP::fixed(TU::core_nested_sequence()).into(),
                Ok(7),
            ),
            (
                VP::raw(vec![Err(Error::Sentinel)]).into(),
                Err(ErrorKind::Sentinel),
            ),
            (
                VP::raw(vec![Ok(MetaVal::Bul(true)), Ok(MetaVal::Bul(true)), Err(Error::Sentinel)]).into(),
                Err(ErrorKind::Sentinel),
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = input.count().map_err(ErrorKind::from);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_first() {
        let inputs_and_expected: Vec<(IL, Result<MetaVal, ErrorKind>)> = vec![
            (
                vec![].into(),
                Err(ErrorKind::EmptySequence),
            ),
            (
                TU::core_nested_sequence().into(),
                Ok(TU::core_nested_sequence()[0].clone()),
            ),
            (
                VP::fixed(vec![]).into(),
                Err(ErrorKind::EmptyProducer),
            ),
            (
                VP::fixed(TU::core_nested_sequence()).into(),
                Ok(TU::core_nested_sequence()[0].clone()),
            ),
            (
                VP::raw(vec![Err(Error::Sentinel), Ok(MetaVal::Bul(true)), Ok(MetaVal::Bul(false))]).into(),
                Err(ErrorKind::Sentinel),
            ),
            (
                VP::raw(vec![Ok(MetaVal::Bul(true)), Ok(MetaVal::Bul(false)), Err(Error::Sentinel)]).into(),
                Ok(MetaVal::Bul(true)),
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = input.first().map_err(ErrorKind::from);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_last() {
        let inputs_and_expected: Vec<(IL, Result<MetaVal, ErrorKind>)> = vec![
            (
                vec![].into(),
                Err(ErrorKind::EmptySequence),
            ),
            (
                TU::core_nested_sequence().into(),
                Ok(TU::core_nested_sequence().pop().unwrap()),
            ),
            (
                VP::fixed(vec![]).into(),
                Err(ErrorKind::EmptyProducer),
            ),
            (
                VP::fixed(TU::core_nested_sequence()).into(),
                Ok(TU::core_nested_sequence().pop().unwrap()),
            ),
            (
                VP::raw(vec![Err(Error::Sentinel), Ok(MetaVal::Bul(true)), Ok(MetaVal::Bul(false))]).into(),
                Err(ErrorKind::Sentinel),
            ),
            (
                VP::raw(vec![Ok(MetaVal::Bul(true)), Ok(MetaVal::Bul(false)), Err(Error::Sentinel)]).into(),
                Err(ErrorKind::Sentinel),
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = input.last().map_err(ErrorKind::from);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_min_in() {
        let inputs_and_expected: Vec<(IL, Result<NumberLike, ErrorKind>)> = vec![
            (
                vec![].into(),
                Err(ErrorKind::EmptySequence),
            ),
            (
                TU::core_number_sequence(2, false, true, false).into(),
                Ok(NumberLike::Integer(-2)),
            ),
            (
                TU::core_number_sequence(2, true, true, false).into(),
                Ok(NumberLike::Decimal(TU::d_raw(-25, 1))),
            ),
            (
                vec![TU::i(1)].into(),
                Ok(NumberLike::Integer(1)),
            ),
            (
                vec![TU::i(1), MetaVal::Bul(true)].into(),
                Err(ErrorKind::NotNumeric),
            ),
            (
                VP::fixed(vec![]).into(),
                Err(ErrorKind::EmptyProducer),
            ),
            (
                VP::fixed(TU::core_number_sequence(2, false, true, false)).into(),
                Ok(NumberLike::Integer(-2)),
            ),
            (
                VP::fixed(TU::core_number_sequence(2, true, true, false)).into(),
                Ok(NumberLike::Decimal(TU::d_raw(-25, 1))),
            ),
            (
                VP::raw(vec![Ok(TU::i(1))]).into(),
                Ok(NumberLike::Integer(1)),
            ),
            (
                VP::raw(vec![Ok(TU::i(1)), Ok(MetaVal::Bul(false))]).into(),
                Err(ErrorKind::NotNumeric),
            ),
            (
                VP::raw(vec![Ok(TU::i(1)), Err(Error::Sentinel)]).into(),
                Err(ErrorKind::Sentinel),
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = input.min_in().map_err(ErrorKind::from);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_max_in() {
        let inputs_and_expected: Vec<(IL, Result<NumberLike, ErrorKind>)> = vec![
            (
                vec![].into(),
                Err(ErrorKind::EmptySequence),
            ),
            (
                TU::core_number_sequence(2, false, true, false).into(),
                Ok(NumberLike::Integer(2)),
            ),
            (
                TU::core_number_sequence(2, true, true, false).into(),
                Ok(NumberLike::Decimal(TU::d_raw(25, 1))),
            ),
            (
                vec![TU::i(1)].into(),
                Ok(NumberLike::Integer(1)),
            ),
            (
                vec![TU::i(1), MetaVal::Bul(true)].into(),
                Err(ErrorKind::NotNumeric),
            ),
            (
                VP::fixed(vec![]).into(),
                Err(ErrorKind::EmptyProducer),
            ),
            (
                VP::fixed(TU::core_number_sequence(2, false, true, false)).into(),
                Ok(NumberLike::Integer(2)),
            ),
            (
                VP::fixed(TU::core_number_sequence(2, true, true, false)).into(),
                Ok(NumberLike::Decimal(TU::d_raw(25, 1))),
            ),
            (
                VP::raw(vec![Ok(TU::i(1))]).into(),
                Ok(NumberLike::Integer(1)),
            ),
            (
                VP::raw(vec![Ok(TU::i(1)), Ok(MetaVal::Bul(false))]).into(),
                Err(ErrorKind::NotNumeric),
            ),
            (
                VP::raw(vec![Ok(TU::i(1)), Err(Error::Sentinel)]).into(),
                Err(ErrorKind::Sentinel),
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = input.max_in().map_err(ErrorKind::from);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_rev() {
        let inputs_and_expected: Vec<(IL, Result<Vec<MetaVal>, ErrorKind>)> = vec![
            (
                vec![].into(),
                Ok(vec![]),
            ),
            (
                TU::core_nested_sequence().into(),
                Ok({ let mut s = TU::core_nested_sequence(); s.reverse(); s }),
            ),
            (
                vec![TU::i(1)].into(),
                Ok(vec![TU::i(1)]),
            ),
            (
                VP::fixed(vec![]).into(),
                Ok(vec![]),
            ),
            (
                VP::fixed(TU::core_nested_sequence()).into(),
                Ok({ let mut s = TU::core_nested_sequence(); s.reverse(); s }),
            ),
            (
                VP::raw(vec![Ok(TU::i(1))]).into(),
                Ok(vec![TU::i(1)]),
            ),
            (
                VP::raw(vec![Ok(TU::i(1)), Err(Error::Sentinel)]).into(),
                Err(ErrorKind::Sentinel),
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = input.rev().map_err(ErrorKind::from);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_sort() {
        let inputs_and_expected: Vec<(IL, Result<Vec<MetaVal>, ErrorKind>)> = vec![
            (
                vec![].into(),
                Ok(vec![]),
            ),
            (
                TU::core_number_sequence(2, false, true, true).into(),
                Ok(vec![TU::i(-2), TU::d(-15, 1), TU::i(-1), TU::d(-5, 1), TU::i(0), TU::d(5, 1), TU::i(1), TU::d(15, 1), TU::i(2)]),
            ),
            (
                vec![TU::i(1)].into(),
                Ok(vec![TU::i(1)]),
            ),
            (
                VP::fixed(vec![]).into(),
                Ok(vec![]),
            ),
            (
                VP::fixed(TU::core_number_sequence(2, false, true, true)).into(),
                Ok(vec![TU::i(-2), TU::d(-15, 1), TU::i(-1), TU::d(-5, 1), TU::i(0), TU::d(5, 1), TU::i(1), TU::d(15, 1), TU::i(2)]),
            ),
            (
                VP::raw(vec![Ok(TU::i(1))]).into(),
                Ok(vec![TU::i(1)]),
            ),
            (
                VP::raw(vec![Ok(TU::i(1)), Err(Error::Sentinel)]).into(),
                Err(ErrorKind::Sentinel),
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = input.sort().map_err(ErrorKind::from);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_sum() {
        let inputs_and_expected: Vec<(IL, Result<NumberLike, ErrorKind>)> = vec![
            (
                vec![].into(),
                Ok(NumberLike::Integer(0)),
            ),
            (
                TU::core_number_sequence(2, false, true, true).into(),
                Ok(NumberLike::Decimal(TU::d_raw(0, 0))),
            ),
            (
                vec![TU::i(-2), TU::i(3), TU::i(5), TU::i(7)].into(),
                Ok(NumberLike::Integer(13)),
            ),
            (
                vec![TU::i(-2), TU::i(3), TU::d(55, 1), TU::i(7)].into(),
                Ok(NumberLike::Decimal(TU::d_raw(135, 1))),
            ),
            (
                vec![TU::i(1)].into(),
                Ok(NumberLike::Integer(1)),
            ),
            (
                vec![TU::i(1), MetaVal::Bul(true)].into(),
                Err(ErrorKind::NotNumeric),
            ),
            (
                VP::fixed(vec![]).into(),
                Ok(NumberLike::Integer(0)),
            ),
            (
                VP::fixed(TU::core_number_sequence(2, false, true, true)).into(),
                Ok(NumberLike::Decimal(TU::d_raw(0, 0))),
            ),
            (
                VP::raw(vec![Ok(TU::i(-2)), Ok(TU::i(3)), Ok(TU::i(5)), Ok(TU::i(7))]).into(),
                Ok(NumberLike::Integer(13)),
            ),
            (
                VP::raw(vec![Ok(TU::i(-2)), Ok(TU::i(3)), Ok(TU::d(55, 1)), Ok(TU::i(7))]).into(),
                Ok(NumberLike::Decimal(TU::d_raw(135, 1))),
            ),
            (
                VP::raw(vec![Ok(TU::i(1))]).into(),
                Ok(NumberLike::Integer(1)),
            ),
            (
                VP::raw(vec![Ok(TU::i(1)), Ok(MetaVal::Bul(true))]).into(),
                Err(ErrorKind::NotNumeric),
            ),
            (
                VP::raw(vec![Ok(TU::i(1)), Err(Error::Sentinel)]).into(),
                Err(ErrorKind::Sentinel),
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = input.sum().map_err(ErrorKind::from);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_prod() {
        let inputs_and_expected: Vec<(IL, Result<NumberLike, ErrorKind>)> = vec![
            (
                vec![].into(),
                Ok(NumberLike::Integer(1)),
            ),
            (
                TU::core_number_sequence(2, false, true, true).into(),
                Ok(NumberLike::Decimal(TU::d_raw(0, 0))),
            ),
            (
                TU::core_number_sequence(2, false, true, false).into(),
                Ok(NumberLike::Decimal(TU::d_raw(225, 2))),
            ),
            (
                vec![TU::i(-2), TU::i(3), TU::i(5), TU::i(7)].into(),
                Ok(NumberLike::Integer(-210)),
            ),
            (
                vec![TU::i(-2), TU::i(3), TU::d(55, 1), TU::i(7)].into(),
                Ok(NumberLike::Decimal(TU::d_raw(-231, 0))),
            ),
            (
                vec![TU::i(1)].into(),
                Ok(NumberLike::Integer(1)),
            ),
            (
                vec![TU::i(1), MetaVal::Bul(true)].into(),
                Err(ErrorKind::NotNumeric),
            ),
            (
                VP::fixed(vec![]).into(),
                Ok(NumberLike::Integer(1)),
            ),
            (
                VP::fixed(TU::core_number_sequence(2, false, true, true)).into(),
                Ok(NumberLike::Decimal(TU::d_raw(0, 0))),
            ),
            (
                VP::fixed(TU::core_number_sequence(2, false, true, false)).into(),
                Ok(NumberLike::Decimal(TU::d_raw(225, 2))),
            ),
            (
                VP::raw(vec![Ok(TU::i(-2)), Ok(TU::i(3)), Ok(TU::i(5)), Ok(TU::i(7))]).into(),
                Ok(NumberLike::Integer(-210)),
            ),
            (
                VP::raw(vec![Ok(TU::i(-2)), Ok(TU::i(3)), Ok(TU::d(55, 1)), Ok(TU::i(7))]).into(),
                Ok(NumberLike::Decimal(TU::d_raw(-231, 0))),
            ),
            (
                VP::raw(vec![Ok(TU::i(1))]).into(),
                Ok(NumberLike::Integer(1)),
            ),
            (
                VP::raw(vec![Ok(TU::i(1)), Ok(MetaVal::Bul(true))]).into(),
                Err(ErrorKind::NotNumeric),
            ),
            (
                VP::raw(vec![Ok(TU::i(1)), Err(Error::Sentinel)]).into(),
                Err(ErrorKind::Sentinel),
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = input.prod().map_err(ErrorKind::from);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_all_equal() {
        let inputs_and_expected: Vec<(IL, Result<bool, ErrorKind>)> = vec![
            (
                vec![].into(),
                Ok(true),
            ),
            (
                vec![TU::i(1), TU::i(1), TU::i(1)].into(),
                Ok(true),
            ),
            (
                vec![TU::i(1), TU::i(1), TU::i(2)].into(),
                Ok(false),
            ),
            (
                vec![TU::i(1)].into(),
                Ok(true),
            ),
            (
                vec![TU::i(1), MetaVal::Bul(true)].into(),
                Ok(false),
            ),
            (
                VP::fixed(vec![]).into(),
                Ok(true),
            ),
            (
                VP::raw(vec![Ok(TU::i(1)), Ok(TU::i(1)), Ok(TU::i(1))]).into(),
                Ok(true),
            ),
            (
                VP::raw(vec![Ok(TU::i(1)), Ok(TU::i(1)), Ok(TU::i(2))]).into(),
                Ok(false),
            ),
            (
                VP::raw(vec![Ok(TU::i(1))]).into(),
                Ok(true),
            ),
            (
                VP::raw(vec![Ok(TU::i(1)), Ok(MetaVal::Bul(true))]).into(),
                Ok(false),
            ),
            (
                VP::raw(vec![Ok(TU::i(1)), Err(Error::Sentinel)]).into(),
                Err(ErrorKind::Sentinel),
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = input.all_equal().map_err(ErrorKind::from);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_flatten() {
        let inputs_and_expected: Vec<(IL, Result<Vec<Result<MetaVal, ErrorKind>>, ErrorKind>)> = vec![
            (
                vec![].into(),
                Ok(vec![]),
            ),
            (
                TU::core_flat_sequence().into(),
                Ok(TU::core_flat_sequence().into_iter().map(Result::Ok).collect()),
            ),
            (
                TU::core_nested_sequence().into(),
                Ok({
                    let mut s = TU::core_flat_sequence();
                    s.extend(TU::core_flat_sequence());
                    s.push(TU::sample_flat_mapping());
                    s
                }.into_iter().map(Result::Ok).collect()),
            ),
            (
                VP::fixed(vec![]).into(),
                Ok(vec![]),
            ),
            (
                VP::fixed(TU::core_flat_sequence()).into(),
                Ok(TU::core_flat_sequence().into_iter().map(Result::Ok).collect()),
            ),
            (
                VP::fixed(TU::core_nested_sequence()).into(),
                Ok({
                    let mut s = TU::core_flat_sequence();
                    s.extend(TU::core_flat_sequence());
                    s.push(TU::sample_flat_mapping());
                    s
                }.into_iter().map(Result::Ok).collect()),
            ),
            (
                VP::raw(vec![Ok(TU::i(1)), Err(Error::Sentinel)]).into(),
                Ok(vec![Ok(TU::i(1)), Err(ErrorKind::Sentinel)]),
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = input.flatten()
                .map_err(ErrorKind::from)
                .map(|il| {
                    il.into_iter().map(|res| {
                        res.map_err(ErrorKind::from)
                    })
                    .collect::<Vec<_>>()
                })
            ;
            assert_eq!(expected, produced);
        }
    }

    // #[test]
    // fn test_dedup() {
    //     let inputs_and_expected = vec![
    //         (
    //             vec![],
    //             vec![],
    //         ),
    //         (
    //             TU::core_flat_sequence().into_iter().map(Result::Ok).collect(),
    //             TU::core_flat_sequence().into_iter().map(Result::Ok).collect(),
    //         ),
    //         (
    //             TU::core_nested_sequence().into_iter().map(Result::Ok).collect(),
    //             TU::core_nested_sequence().into_iter().map(Result::Ok).collect(),
    //         ),
    //         (
    //             vec![Ok(TU::i(1)), Ok(TU::i(1)), Ok(TU::i(1)), Ok(TU::i(2)), Ok(TU::i(2)), Ok(TU::i(3)), Ok(TU::i(3)), Ok(TU::i(3)), Ok(TU::i(1))],
    //             vec![Ok(TU::i(1)), Ok(TU::i(2)), Ok(TU::i(3)), Ok(TU::i(1))],
    //         ),
    //         (
    //             vec![Ok(TU::i(1)), Ok(TU::i(2)), Ok(TU::i(3)), Ok(TU::i(4)), Ok(TU::i(5))],
    //             vec![Ok(TU::i(1)), Ok(TU::i(2)), Ok(TU::i(3)), Ok(TU::i(4)), Ok(TU::i(5))],
    //         ),
    //         (
    //             vec![Ok(TU::i(1)), Ok(TU::i(1)), Ok(TU::i(1)), Ok(TU::i(1)), Ok(TU::i(1))],
    //             vec![Ok(TU::i(1))],
    //         ),
    //         (
    //             vec![Ok(TU::i(1)), Err(Error::Sentinel)],
    //             vec![Ok(TU::i(1)), Err(ErrorKind::Sentinel)],
    //         ),
    //     ];

    //     for (input, expected) in inputs_and_expected {
    //         let produced = input.dedup().map(|e| e.map_err(ErrorKind::from)).collect::<Vec<_>>();
    //         assert_eq!(expected, produced);
    //     }
    // }

    // #[test]
    // fn test_dedup_s() {
    //     let inputs_and_expected = vec![
    //         (
    //             vec![],
    //             vec![],
    //         ),
    //         (
    //             TU::core_flat_sequence(),
    //             TU::core_flat_sequence(),
    //         ),
    //         (
    //             TU::core_nested_sequence(),
    //             TU::core_nested_sequence(),
    //         ),
    //         (
    //             vec![TU::i(1), TU::i(1), TU::i(1), TU::i(2), TU::i(2), TU::i(3), TU::i(3), TU::i(3), TU::i(1)],
    //             vec![TU::i(1), TU::i(2), TU::i(3), TU::i(1)],
    //         ),
    //         (
    //             vec![TU::i(1), TU::i(2), TU::i(3), TU::i(4), TU::i(5)],
    //             vec![TU::i(1), TU::i(2), TU::i(3), TU::i(4), TU::i(5)],
    //         ),
    //         (
    //             vec![TU::i(1), TU::i(1), TU::i(1), TU::i(1), TU::i(1)],
    //             vec![TU::i(1)],
    //         ),
    //     ];

    //     for (input, expected) in inputs_and_expected {
    //         let produced = input.dedup_s(input);
    //         assert_eq!(expected, produced);
    //     }
    // }

    // #[test]
    // fn test_unique() {
    //     let inputs_and_expected = vec![
    //         (
    //             vec![],
    //             vec![],
    //         ),
    //         (
    //             TU::core_flat_sequence().into_iter().map(Result::Ok).collect(),
    //             TU::core_flat_sequence().into_iter().map(Result::Ok).collect(),
    //         ),
    //         (
    //             TU::core_nested_sequence().into_iter().map(Result::Ok).collect(),
    //             TU::core_nested_sequence().into_iter().map(Result::Ok).collect(),
    //         ),
    //         (
    //             vec![Ok(TU::i(1)), Ok(TU::i(1)), Ok(TU::i(1)), Ok(TU::i(2)), Ok(TU::i(2)), Ok(TU::i(3)), Ok(TU::i(3)), Ok(TU::i(3)), Ok(TU::i(1))],
    //             vec![Ok(TU::i(1)), Ok(TU::i(2)), Ok(TU::i(3))],
    //         ),
    //         (
    //             vec![Ok(TU::i(1)), Ok(TU::i(2)), Ok(TU::i(3)), Ok(TU::i(3)), Ok(TU::i(2)), Ok(TU::i(1))],
    //             vec![Ok(TU::i(1)), Ok(TU::i(2)), Ok(TU::i(3))],
    //         ),
    //         (
    //             vec![Ok(TU::i(1)), Ok(TU::i(2)), Ok(TU::i(3)), Ok(TU::i(4)), Ok(TU::i(5))],
    //             vec![Ok(TU::i(1)), Ok(TU::i(2)), Ok(TU::i(3)), Ok(TU::i(4)), Ok(TU::i(5))],
    //         ),
    //         (
    //             vec![Ok(TU::i(1)), Ok(TU::i(1)), Ok(TU::i(1)), Ok(TU::i(1)), Ok(TU::i(1))],
    //             vec![Ok(TU::i(1))],
    //         ),
    //         (
    //             vec![Ok(TU::i(1)), Err(Error::Sentinel)],
    //             vec![Ok(TU::i(1)), Err(ErrorKind::Sentinel)],
    //         ),
    //     ];

    //     for (input, expected) in inputs_and_expected {
    //         let produced = input.unique().map(|e| e.map_err(ErrorKind::from)).collect::<Vec<_>>();
    //         assert_eq!(expected, produced);
    //     }
    // }

    // #[test]
    // fn test_unique_s() {
    //     let inputs_and_expected = vec![
    //         (
    //             vec![],
    //             vec![],
    //         ),
    //         (
    //             TU::core_flat_sequence(),
    //             TU::core_flat_sequence(),
    //         ),
    //         (
    //             TU::core_nested_sequence(),
    //             TU::core_nested_sequence(),
    //         ),
    //         (
    //             vec![TU::i(1), TU::i(1), TU::i(1), TU::i(2), TU::i(2), TU::i(3), TU::i(3), TU::i(3), TU::i(1)],
    //             vec![TU::i(1), TU::i(2), TU::i(3)],
    //         ),
    //         (
    //             vec![TU::i(1), TU::i(2), TU::i(3), TU::i(3), TU::i(2), TU::i(1)],
    //             vec![TU::i(1), TU::i(2), TU::i(3)],
    //         ),
    //         (
    //             vec![TU::i(1), TU::i(2), TU::i(3), TU::i(4), TU::i(5)],
    //             vec![TU::i(1), TU::i(2), TU::i(3), TU::i(4), TU::i(5)],
    //         ),
    //         (
    //             vec![TU::i(1), TU::i(1), TU::i(1), TU::i(1), TU::i(1)],
    //             vec![TU::i(1)],
    //         ),
    //     ];

    //     for (input, expected) in inputs_and_expected {
    //         let produced = input.unique_s(input);
    //         assert_eq!(expected, produced);
    //     }
    // }
}
