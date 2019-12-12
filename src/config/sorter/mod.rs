pub mod sort_by;
pub mod sort_order;

use std::path::Path;
use std::cmp::Ordering;

use self::sort_by::SortBy;
use self::sort_order::SortOrder;

pub struct Sorter {
    sort_by: SortBy,
    sort_order: SortOrder,
}

impl Sorter {
    pub fn path_sort_cmp<P: AsRef<Path>>(&self, abs_item_path_a: P, abs_item_path_b: P) -> Ordering {
        let ordering = self.sort_by.path_sort_cmp(abs_item_path_a, abs_item_path_b);

        match self.sort_order {
            SortOrder::Ascending => ordering,
            SortOrder::Descending => ordering.reverse(),
        }
    }
}
