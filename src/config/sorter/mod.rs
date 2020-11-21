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

    /// Compares two `Result`s containing absolute item paths using this
    /// sorting criteria.
    pub fn cmp_path_results<P, E>(&self, res_a: &Result<P, E>, res_b: &Result<P, E>) -> Ordering
    where
        P: AsRef<Path>,
    {
        // TODO: Might be better to always sort `Error`s to the front,
        //       regardless of sort direction.
        self.align(self.sort_by.cmp_path_results(res_a, res_b))
    }

    pub fn sort_paths<P>(&self, paths: &mut [P])
    where
        P: AsRef<Path>,
    {
        paths.sort_by(|a, b| self.cmp_paths(a, b));
    }

    pub fn sort_path_results<P, E>(&self, path_results: &mut [Result<P, E>])
    where
        P: AsRef<Path>,
    {
        path_results.sort_by(|a, b| self.cmp_path_results(a, b));
    }
}
