use std::path::Path;
use std::cmp::Ordering;
use std::time::SystemTime;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SortOrder {
    Name,
    ModTime,
}

impl SortOrder {
    pub fn path_sort_cmp<P: AsRef<Path>>(&self, abs_item_path_a: P, abs_item_path_b: P) -> Ordering {
        let abs_item_path_a = abs_item_path_a.as_ref();
        let abs_item_path_b = abs_item_path_b.as_ref();

        match *self {
            SortOrder::Name => abs_item_path_a.file_name().cmp(&abs_item_path_b.file_name()),
            SortOrder::ModTime => SortOrder::get_mtime(abs_item_path_a).cmp(&SortOrder::get_mtime(abs_item_path_b)),
        }
    }

    fn get_mtime<P: AsRef<Path>>(abs_path: P) -> Option<SystemTime> {
        abs_path.as_ref().metadata().and_then(|m| m.modified()).ok()
    }
}
