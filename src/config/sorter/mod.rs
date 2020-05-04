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
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct Sorter {
    pub sort_by: SortBy,
    pub sort_order: SortOrder,
}

impl Sorter {
    /// Compares two absolute item file paths using this sorting criteria.
    pub fn path_sort_cmp(&self, abs_path_a: &Path, abs_path_b: &Path) -> Ordering {
        let ordering = self.sort_by.path_sort_cmp(abs_path_a, abs_path_b);

        match self.sort_order {
            SortOrder::Ascending => ordering,
            SortOrder::Descending => ordering.reverse(),
        }
    }
}
