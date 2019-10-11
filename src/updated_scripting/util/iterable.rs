
use std::borrow::Cow;

use crate::metadata::types::MetaVal;
use crate::updated_scripting::Error;
use crate::updated_scripting::util::Util;

#[derive(Copy, Clone)]
enum RevSort { Rev, Sort, }

/// Represents one of several different kinds of iterables, producing meta values.
pub enum Iterable<'a> {
    Borrowed(&'a [MetaVal]),
    Owned(Vec<MetaVal>),
    // Producer(ValueProducer<'a>),
}

impl<'a> Iterator for Iterable<'a> {
    type Item = Result<Cow<'a, MetaVal>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

impl<'a> Iterable<'a> {
    pub fn is_lazy(&self) -> bool {
        match self {
            &Self::Borrowed(..) => false,
            &Self::Owned(..) => false,
            // &Self::Producer(..) => true,
        }
    }

    /// Counts the number of items contained in this iterable.
    pub fn count(self) -> usize {
        match self {
            Self::Borrowed(s) => s.len(),
            Self::Owned(s) => s.len(),
        }
    }

    /// Collects the contained items eagerly.
    /// This is a no-op if this iterable is already collected.
    pub fn collect(self) -> Vec<MetaVal> {
        match self {
            Self::Borrowed(s) => s.to_vec(),
            Self::Owned(s) => s,
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
            Self::Borrowed(s) => s.first().map(Cow::Borrowed),
            Self::Owned(s) => s.into_iter().next().map(Cow::Owned),
        }
    }

    /// Returns the last item in this iterable, if there is one.
    pub fn last(self) -> Option<Cow<'a, MetaVal>> {
        match self {
            Self::Borrowed(s) => s.last().map(Cow::Borrowed),
            Self::Owned(s) => s.into_iter().last().map(Cow::Owned),
        }
    }

    /// Checks if all items are equal to each other.
    /// If empty, returns true.
    pub fn all_equal(self) -> bool {
        true
    }
}
