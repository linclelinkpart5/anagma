
use std::borrow::Cow;
use std::convert::TryInto;

use crate::util::Number;
use crate::metadata::types::MetaVal;
use crate::updated_scripting::Error;
use crate::updated_scripting::util::Util;
use crate::updated_scripting::util::IteratorLike;
use crate::updated_scripting::util::Producer;
use crate::updated_scripting::util::producer::Fixed;
use crate::updated_scripting::util::producer::Filter;
use crate::updated_scripting::util::producer::Map;
use crate::updated_scripting::util::producer::StepBy;
use crate::updated_scripting::util::producer::Chain;
use crate::updated_scripting::traits::Predicate;
use crate::updated_scripting::traits::Converter;

#[derive(Copy, Clone)]
enum RevSort { Rev, Sort, }

#[derive(Clone, Copy)]
enum MinMax { Min, Max, }

#[derive(Clone, Copy)]
enum SumProd { Sum, Prod, }

#[derive(Clone, Copy)]
enum AllAny { All, Any, }

/// Represents one of several different kinds of iterables, producing meta values.
pub enum IterableLike<'a> {
    Slice(&'a [MetaVal]),
    Vector(Vec<MetaVal>),
    Producer(Producer),
}

impl<'a> IntoIterator for IterableLike<'a> {
    type Item = Result<Cow<'a, MetaVal>, Error>;
    type IntoIter = IteratorLike<'a>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Slice(s) => IteratorLike::Slice(s.into_iter()),
            Self::Vector(v) => IteratorLike::Vector(v.into_iter()),
            Self::Producer(p) => IteratorLike::Producer(p),
        }
    }
}

impl<'a> IterableLike<'a> {
    fn is_lazy(&self) -> bool {
        match self {
            &Self::Slice(..) => false,
            &Self::Vector(..) => false,
            &Self::Producer(..) => true,
        }
    }

    fn into_producer(self) -> (Producer, bool) {
        let is_lazy = self.is_lazy();

        let producer = match self {
            Self::Slice(s) => Producer::new(Fixed::new(s.to_vec())),
            Self::Vector(v) => Producer::new(Fixed::new(v)),
            Self::Producer(p) => p,
        };

        (producer, is_lazy)
    }

    /// Collects the contained items eagerly.
    /// This is a no-op if this iterable is already collected.
    pub fn collect(self) -> Result<Vec<MetaVal>, Error> {
        match self {
            Self::Slice(s) => Ok(s.to_vec()),
            Self::Vector(v) => Ok(v),
            Self::Producer(p) => p.collect(),
        }
    }

    /// Helper method for `rev`/`sort`.
    fn rev_sort(self, flag: RevSort) -> Result<Vec<MetaVal>, Error> {
        let mut seq = self.collect()?;

        match flag {
            // Reverse the slice mutably in-place.
            RevSort::Rev => seq.reverse(),

            // Sort the slice mutably in-place, using the default sort comparison.
            RevSort::Sort => seq.sort_by(Util::default_sort_by),
        };

        Ok(seq)
    }

    /// Reverses the order of the items in this iterable.
    /// Eagerly collects the items beforehand if not already collected.
    pub fn rev(self) -> Result<Vec<MetaVal>, Error> {
        self.rev_sort(RevSort::Rev)
    }

    /// Sorts the items in this iterable using the default sort comparison.
    /// Eagerly collects the items beforehand if not already collected.
    pub fn sort(self) -> Result<Vec<MetaVal>, Error> {
        self.rev_sort(RevSort::Sort)
    }

    /// Counts the number of items contained in this iterable.
    pub fn count(self) -> Result<usize, Error> {
        match self {
            Self::Slice(s) => Ok(s.len()),
            Self::Vector(v) => Ok(v.len()),
            Self::Producer(p) => p.len(),
        }
    }

    /// Returns the first item in this iterable, if there is one.
    pub fn first(self) -> Result<Option<Cow<'a, MetaVal>>, Error> {
        match self {
            Self::Slice(s) => Ok(s.first().map(Cow::Borrowed)),
            Self::Vector(v) => Ok(v.into_iter().next().map(Cow::Owned)),
            Self::Producer(p) => p.first().map(|opt| opt.map(Cow::Owned)),
        }
    }

    /// Returns the last item in this iterable, if there is one.
    pub fn last(self) -> Result<Option<Cow<'a, MetaVal>>, Error> {
        match self {
            Self::Slice(s) => Ok(s.last().map(Cow::Borrowed)),
            Self::Vector(v) => Ok(v.into_iter().last().map(Cow::Owned)),
            Self::Producer(p) => p.last().map(|opt| opt.map(Cow::Owned)),
        }
    }

    /// Helper method for `min_in`/`max_in`.
    fn min_in_max_in(self, flag: MinMax) -> Result<Option<Number>, Error> {
        let mut it = self.into_iter();
        match it.next() {
            // No items, so no min or max.
            None => Ok(None),

            Some(res_first_item) => {
                let first_item = res_first_item?;
                let mut target_num: Number = first_item.as_ref().try_into().map_err(|_| Error::NotNumeric)?;

                for res_item in it {
                    let item = res_item?;
                    let num: Number = item.as_ref().try_into().map_err(|_| Error::NotNumeric)?;
                    target_num = match flag {
                        MinMax::Min => target_num.val_min(num),
                        MinMax::Max => target_num.val_max(num),
                    };
                }

                Ok(Some(target_num))
            }
        }
    }

    /// Returns the minimum number in this iterable, using the default numerical comparison.
    /// Returns an error if any non-numeric items are found.
    pub fn min_in(self) -> Result<Option<Number>, Error> {
        self.min_in_max_in(MinMax::Min)
    }

    /// Returns the maximum number in this iterable, using the default numerical comparison.
    /// Returns an error if any non-numeric items are found.
    pub fn max_in(self) -> Result<Option<Number>, Error> {
        self.min_in_max_in(MinMax::Max)
    }

    /// Helper method for `sum`/`prod`.
    fn sum_prod(self, flag: SumProd) -> Result<Number, Error> {
        let mut total = match flag {
            SumProd::Sum => Number::Integer(0),
            SumProd::Prod => Number::Integer(1),
        };

        for res_item in self {
            let item = res_item?;
            let num: Number = item.as_ref().try_into().map_err(|_| Error::NotNumeric)?;

            match flag {
                SumProd::Sum => { total = total + num; },
                SumProd::Prod => { total = total * num; },
            };
        }

        Ok(total)
    }

    /// Sums the numbers in this iterable.
    /// Returns an error if any non-numeric items are found.
    pub fn sum(self) -> Result<Number, Error> {
        self.sum_prod(SumProd::Sum)
    }

    /// Multiplies the numbers in this iterable.
    /// Returns an error if any non-numeric items are found.
    pub fn prod(self) -> Result<Number, Error> {
        self.sum_prod(SumProd::Prod)
    }

    /// Checks if all items are equal to each other.
    /// If empty, returns true.
    pub fn all_equal(self) -> Result<bool, Error> {
        let mut it = self.into_iter();
        match it.next() {
            None => Ok(true),
            Some(res_first_item) => {
                let first_item = res_first_item?;
                for res_item in it { if res_item? != first_item { return Ok(false) } }
                Ok(true)
            },
        }
    }

    /// Checks if the iterable has no items.
    /// If empty, returns true.
    pub fn is_empty(self) -> Result<bool, Error> {
        match self {
            Self::Slice(s) => Ok(s.is_empty()),
            Self::Vector(v) => Ok(v.is_empty()),
            Self::Producer(mut p) => {
                match p.next() {
                    None => Ok(true),
                    Some(Ok(_)) => Ok(false),
                    Some(Err(err)) => Err(err),
                }
            },
        }
    }

    // pub fn flatten(self) -> Result<Self, Error> {
    //     Ok(match self {
    //         Self::Sequence(s) => Self::Sequence(Flatten::new(s.into()).collect::<Result<Vec<_>, _>>()?),
    //         Self::Producer(p) => Self::Producer(ValueProducer::Flatten(Flatten::new(p))),
    //     })
    // }

    // pub fn dedup(self) -> Result<Self, Error> {
    //     Ok(match self {
    //         Self::Sequence(s) => Self::Sequence(Dedup::new(s.into()).collect::<Result<Vec<_>, _>>()?),
    //         Self::Producer(p) => Self::Producer(ValueProducer::Dedup(Dedup::new(p))),
    //     })
    // }

    // pub fn unique(self) -> Result<Self, Error> {
    //     Ok(match self {
    //         Self::Sequence(s) => Self::Sequence(Unique::new(s.into()).collect::<Result<Vec<_>, _>>()?),
    //         Self::Producer(p) => Self::Producer(ValueProducer::Unique(Unique::new(p))),
    //     })
    // }

    /// Returns the item at a specific index position in the iterable, if present.
    pub fn nth(self, n: usize) -> Result<Option<Cow<'a, MetaVal>>, Error> {
        match self {
            Self::Slice(s) => Ok(s.get(n).map(Cow::Borrowed)),
            Self::Vector(v) => Ok(v.into_iter().nth(n).map(Cow::Owned)),
            Self::Producer(p) => p.nth(n).map(|opt| opt.map(Cow::Owned)),
        }
    }

    /// Helper method for `all`/`any`.
    fn all_any<P: Predicate>(self, pred: P, flag: AllAny) -> Result<bool, Error> {
        let target = match flag {
            AllAny::All => false,
            AllAny::Any => true,
        };

        for res_item in self {
            let item = res_item?;
            if pred.test(&item)? == target { return Ok(target) }
        }

        Ok(!target)
    }

    /// Applies a predicate to each item in the iterable.
    /// Returns false if the iterable contains an item for which the predicate returns false.
    pub fn all<P: Predicate>(self, pred: P) -> Result<bool, Error> {
        self.all_any(pred, AllAny::All)
    }

    /// Applies a predicate to each item in the iterable.
    /// Returns true if the iterable contains an item for which the predicate returns true.
    pub fn any<P: Predicate>(self, pred: P) -> Result<bool, Error> {
        self.all_any(pred, AllAny::Any)
    }

    /// Helper method for `find`/`position`.
    fn find_position<P: Predicate>(self, pred: P) -> Result<Option<(usize, Cow<'a, MetaVal>)>, Error> {
        for (n, res_item) in self.into_iter().enumerate() {
            let item = res_item?;
            if pred.test(&item)? { return Ok(Some((n, item))) }
        }

        Ok(None)
    }

    /// Finds the first item in the iterable that passes a predicate, and returns the item.
    /// If no items pass the predicate, returns `None`.
    pub fn find<P: Predicate>(self, pred: P) -> Result<Option<Cow<'a, MetaVal>>, Error> {
        Ok(self.find_position(pred)?.map(|(_, item)| item))
    }

    /// Finds the first item in the iterable that passes a predicate, and returns the index of the item.
    /// If no items pass the predicate, returns `None`.
    pub fn position<P: Predicate>(self, pred: P) -> Result<Option<usize>, Error> {
        Ok(self.find_position(pred)?.map(|(index, _)| index))
    }

    /// Produces a new iterable containing only items that pass a given predicate.
    pub fn filter<P: Predicate + 'static>(self, pred: P) -> Result<Self, Error> {
        let (inner, is_lazy) = self.into_producer();

        let producer = Filter::new(inner, pred);

        if is_lazy { Ok(Self::Producer(Producer::new(producer))) }
        else { Ok(Self::Vector(producer.collect::<Result<Vec<_>, _>>()?)) }
    }

    /// Produces a new iterable by applying a converter to each item in the original iterable.
    pub fn map<C: Converter + 'static>(self, conv: C) -> Result<Self, Error> {
        let (inner, is_lazy) = self.into_producer();

        let producer = Map::new(inner, conv);

        if is_lazy { Ok(Self::Producer(Producer::new(producer))) }
        else { Ok(Self::Vector(producer.collect::<Result<Vec<_>, _>>()?)) }
    }

    /// Produces a new iterable by skipping a fixed number of items from the original iterable after each item.
    pub fn step_by(self, step: usize) -> Result<Self, Error> {
        let (inner, is_lazy) = self.into_producer();

        let producer = StepBy::new(inner, step);

        if is_lazy { Ok(Self::Producer(Producer::new(producer))) }
        else { Ok(Self::Vector(producer.collect::<Result<Vec<_>, _>>()?)) }
    }

    /// Produces a new iterable by concatenating ("chaining") together this iterable with another.
    pub fn chain(self, iter: Self) -> Result<Self, Error> {
        let (inner_a, is_lazy_a) = self.into_producer();
        let (inner_b, is_lazy_b) = iter.into_producer();
        let is_lazy = is_lazy_a || is_lazy_b;

        let producer = Chain::new(inner_a, inner_b);

        if is_lazy { Ok(Self::Producer(Producer::new(producer))) }
        else { Ok(Self::Vector(producer.collect::<Result<Vec<_>, _>>()?)) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rand::Rng;

    use crate::test_util::TestUtil as TU;

    struct TestPredicate(String);

    impl Predicate for TestPredicate {
        fn test(&self, mv: &MetaVal) -> Result<bool, Error> {
            Ok(!match mv {
                &MetaVal::Str(ref s) => &self.0 == s,
                _ => false,
            })
        }
    }

    const CHARS: &[u8] = b"abcde";
    const STR_LEN: usize = 6;

    fn random_string() -> String {
        let mut rng = rand::thread_rng();
        let idx = rng.gen_range(0, CHARS.len());

        (0..STR_LEN).map(|_| CHARS[idx] as char).collect()
    }

    #[test]
    fn test_filter() {
        let mvs = (0..10).map(|_| TU::s(random_string())).collect::<Vec<_>>();

        let target = random_string();

        println!("String to filter out: {}", target);

        println!("Initial:");
        for x in mvs.clone() {
            println!("{:?}", x);
        }

        let il = IterableLike::Producer(Producer::from(mvs.clone()));

        println!("Producer:");
        for x in il.filter(TestPredicate(String::from(target.clone()))).unwrap() {
            println!("{:?}", x);
        }

        let il = IterableLike::Vector(mvs.clone());

        println!("Vector:");
        for x in il.filter(TestPredicate(String::from(target.clone()))).unwrap() {
            println!("{:?}", x);
        }
    }
}
