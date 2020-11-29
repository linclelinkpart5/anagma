//! Defines item file sorting order.

pub mod sort_by;

use std::path::Path;
use std::cmp::Ordering;

use serde::Deserialize;

pub use self::sort_by::SortBy;

/// Represents direction of ordering: ascending or descending.
#[derive(Debug, Copy, Clone, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl Default for SortOrder {
    fn default() -> Self {
        Self::Ascending
    }
}

/// A struct that contains all of the information needed to sort item file paths
/// in a desired order.
#[derive(Debug, Copy, Clone, Deserialize, PartialEq, Eq, Hash, Default)]
#[serde(default, deny_unknown_fields)]
pub struct Sorter {
    pub sort_by: SortBy,
    pub sort_order: SortOrder,
}

impl Sorter {
    fn align(&self, asc_ord: Ordering) -> Ordering {
        match self.sort_order {
            SortOrder::Ascending => asc_ord,
            SortOrder::Descending => asc_ord.reverse(),
        }
    }

    /// Compares two absolute item paths using this sorting criteria.
    pub fn cmp_paths<P>(&self, abs_path_a: &P, abs_path_b: &P) -> Ordering
    where
        P: AsRef<Path>,
    {
        self.align(self.sort_by.cmp_paths(abs_path_a, abs_path_b))
    }

    pub fn sort_paths<P>(&self, paths: &mut [P])
    where
        P: AsRef<Path>,
    {
        paths.sort_by(|a, b| self.cmp_paths(a, b));
    }

    pub fn partition_sort_results<I, P, E>(&self, res_path_iter: I) -> (Vec<E>, Vec<P>)
    where
        I: IntoIterator<Item = Result<P, E>>,
        P: AsRef<Path>,
    {
        let mut errs = Vec::new();
        let mut paths = Vec::new();

        for res_path in res_path_iter {
            match res_path {
                Err(err) => { errs.push(err); },
                Ok(path) => { paths.push(path); }
            }
        }

        self.sort_paths(&mut paths);

        (errs, paths)
    }
}
