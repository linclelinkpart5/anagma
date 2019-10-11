
use std::borrow::Cow;
use std::convert::TryInto;

use crate::util::Number;
use crate::metadata::types::MetaVal;
use crate::updated_scripting::Error;
use crate::updated_scripting::util::Util;
use crate::updated_scripting::util::IteratorLike;

#[derive(Copy, Clone)]
enum RevSort { Rev, Sort, }

#[derive(Clone, Copy)]
enum MinMax { Min, Max, }

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

    /// Returns the minimum number in this list, using the default numerical comparison.
    /// If any non-numeric items are found, returns an error.
    pub fn min_in(self) -> Result<Option<Number>, Error> {
        self.min_in_max_in(MinMax::Min)
    }

    /// Returns the maximum number in this list, using the default numerical comparison.
    /// If any non-numeric items are found, returns an error.
    pub fn max_in(self) -> Result<Option<Number>, Error> {
        self.min_in_max_in(MinMax::Max)
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
}
