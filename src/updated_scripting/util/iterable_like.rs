
use std::borrow::Cow;
use std::convert::TryInto;

use crate::util::Number;
use crate::metadata::types::MetaVal;
use crate::updated_scripting::Error;
use crate::updated_scripting::util::Util;
use crate::updated_scripting::util::IteratorLike;
use crate::updated_scripting::traits::Predicate;

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
    // Producer(ValueProducer<'a>),
}

impl<'a> IntoIterator for IterableLike<'a> {
    type Item = Cow<'a, MetaVal>;
    type IntoIter = IteratorLike<'a>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Slice(s) => IteratorLike::Slice(s.into_iter()),
            Self::Vector(v) => IteratorLike::Vector(v.into_iter()),
        }
    }
}

impl<'a> IterableLike<'a> {
    pub fn is_lazy(&self) -> bool {
        match self {
            &Self::Slice(..) => false,
            &Self::Vector(..) => false,
            // &Self::Producer(..) => true,
        }
    }

    /// Collects the contained items eagerly.
    /// This is a no-op if this iterable is already collected.
    pub fn collect(self) -> Vec<MetaVal> {
        match self {
            Self::Slice(s) => s.to_vec(),
            Self::Vector(s) => s,
        }
    }

    /// Helper method for `rev`/`sort`.
    fn rev_sort(self, flag: RevSort) -> Vec<MetaVal> {
        let mut seq = self.collect();

        match flag {
            // Reverse the slice mutably in-place.
            RevSort::Rev => seq.reverse(),

            // Sort the slice mutably in-place, using the default sort comparison.
            RevSort::Sort => seq.sort_by(Util::default_sort_by),
        };

        seq
    }

    /// Reverses the order of the items in this iterable.
    /// Eagerly collects the items beforehand if not already collected.
    pub fn rev(self) -> Vec<MetaVal> {
        self.rev_sort(RevSort::Rev)
    }

    /// Sorts the items in this iterable using the default sort comparison.
    /// Eagerly collects the items beforehand if not already collected.
    pub fn sort(self) -> Vec<MetaVal> {
        self.rev_sort(RevSort::Sort)
    }

    /// Counts the number of items contained in this iterable.
    pub fn count(self) -> usize {
        match self {
            Self::Slice(s) => s.len(),
            Self::Vector(s) => s.len(),
        }
    }

    /// Returns the first item in this iterable, if there is one.
    pub fn first(self) -> Option<Cow<'a, MetaVal>> {
        match self {
            Self::Slice(s) => s.first().map(Cow::Borrowed),
            Self::Vector(s) => s.into_iter().next().map(Cow::Owned),
        }
    }

    /// Returns the last item in this iterable, if there is one.
    pub fn last(self) -> Option<Cow<'a, MetaVal>> {
        match self {
            Self::Slice(s) => s.last().map(Cow::Borrowed),
            Self::Vector(s) => s.into_iter().last().map(Cow::Owned),
        }
    }

    /// Helper method for `min_in`/`max_in`.
    fn min_in_max_in(self, flag: MinMax) -> Result<Option<Number>, Error> {
        let mut it = self.into_iter();
        match it.next() {
            // No items, so no min or max.
            None => Ok(None),

            Some(first_item) => {
                let mut target_num: Number = first_item.as_ref().try_into().map_err(|_| Error::NotNumeric)?;

                for item in it {
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

        for item in self {
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
    pub fn all_equal(self) -> bool {
        let mut it = self.into_iter();
        match it.next() {
            None => true,
            Some(first_item) => {
                for item in it { if item != first_item { return false } }
                true
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
    pub fn nth(self, n: usize) -> Option<Cow<'a, MetaVal>> {
        match self {
            Self::Slice(s) => s.get(n).map(Cow::Borrowed),
            Self::Vector(v) => v.into_iter().nth(n).map(Cow::Owned),
        }
    }

    /// Helper method for `all`/`any`.
    fn all_any<P: Predicate>(self, pred: P, flag: AllAny) -> bool {
        let target = match flag {
            AllAny::All => false,
            AllAny::Any => true,
        };

        for item in self { if pred.test(&item) == target { return target } }

        !target
    }

    /// Applies a predicate to each item in the iterable.
    /// Returns false if the iterable contains an item for which the predicate returns false.
    pub fn all<P: Predicate>(self, pred: P) -> bool {
        self.all_any(pred, AllAny::All)
    }

    /// Applies a predicate to each item in the iterable.
    /// Returns true if the iterable contains an item for which the predicate returns true.
    pub fn any<P: Predicate>(self, pred: P) -> bool {
        self.all_any(pred, AllAny::Any)
    }
}
