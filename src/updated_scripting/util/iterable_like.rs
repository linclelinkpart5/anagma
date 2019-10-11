
use std::borrow::Cow;

use crate::metadata::types::MetaVal;
use crate::updated_scripting::util::Util;
use crate::updated_scripting::util::iterator_like::IteratorLike;

#[derive(Copy, Clone)]
enum RevSort { Rev, Sort, }

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

    /// Counts the number of items contained in this iterable.
    pub fn count(self) -> usize {
        match self {
            Self::Slice(s) => s.len(),
            Self::Vector(s) => s.len(),
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
    fn rev_sort(seq: Vec<MetaVal>, flag: RevSort) -> Vec<MetaVal> {
        let mut seq = seq;

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
        Self::rev_sort(self.collect(), RevSort::Rev)
    }

    /// Sorts the items in this iterable using the default sort comparison.
    /// Eagerly collects the items beforehand if not already collected.
    pub fn sort(self) -> Vec<MetaVal> {
        Self::rev_sort(self.collect(), RevSort::Sort)
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

    /// Checks if all items are equal to each other.
    /// If empty, returns true.
    pub fn all_equal(self) -> bool {
        let mut it = self.into_iter();
        match it.next() {
            None => true,
            Some(first_item) => {
                for item in it {
                    if item != first_item { return false }
                }

                true
            },
        }
    }
}
