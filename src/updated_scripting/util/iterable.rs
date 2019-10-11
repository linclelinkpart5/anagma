
use crate::metadata::types::MetaVal;

use crate::updated_scripting::util::Util;

#[derive(Copy, Clone)]
enum RevSort { Rev, Sort, }

/// Represents one of several different kinds of iterables, producing meta values.
pub enum Iterable<'a> {
    Borrowed(&'a [MetaVal]),
    Owned(Vec<MetaVal>),
    // Producer(ValueProducer<'a>),
}

impl<'a> Iterable<'a> {
    pub fn is_lazy(&self) -> bool {
        match self {
            &Self::Borrowed(..) => false,
            &Self::Owned(..) => false,
            // &Self::Producer(..) => true,
        }
    }

    pub fn is_eager(&self) -> bool {
        !self.is_lazy()
    }

    /// Counts the number of values contained in this iterable.
    pub fn count(self) -> usize {
        match self {
            Self::Borrowed(s) => s.len(),
            Self::Owned(s) => s.len(),
        }
    }

    /// Collects the contained values eagerly.
    /// This is a no-op if this iterable is already collected.
    pub fn collect(self) -> Vec<MetaVal> {
        match self {
            Self::Borrowed(s) => s.to_vec(),
            Self::Owned(s) => s,
        }
    }

    /// Helper method for `rev`/`sort`.
    fn rev_sort(mut seq: Vec<MetaVal>, flag: RevSort) -> Vec<MetaVal> {
        match flag {
            // Reverse the slice mutably in-place.
            RevSort::Rev => seq.reverse(),

            // Sort the slice mutably in-place, using the default sort comparison.
            RevSort::Sort => seq.sort_by(Util::default_sort_by),
        };

        seq
    }

    /// Reverses the order of the values in this iterable.
    /// Eagerly collects the values beforehand if not already collected.
    pub fn rev(self) -> Vec<MetaVal> {
        Self::rev_sort(self.collect(), RevSort::Rev)
    }

    /// Sorts the values in this iterable using the default sort comparison.
    /// Eagerly collects the values beforehand if not already collected.
    pub fn sort(self) -> Vec<MetaVal> {
        Self::rev_sort(self.collect(), RevSort::Sort)
    }
}
