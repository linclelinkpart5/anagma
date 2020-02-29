use std::path::Path;
use std::cmp::Ordering;

use serde::Deserialize;

use crate::util::Util;

/// Represents all criteria that can be used for sorting item files.
#[derive(Debug, Copy, Clone, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum SortBy {
    Name,
    ModTime,
}

impl SortBy {
    /// Compares two absolute item file paths using this sorting criteria.
    pub fn path_sort_cmp<P: AsRef<Path>>(
        &self,
        abs_item_path_a: P,
        abs_item_path_b: P
    ) -> Ordering
    {
        let abs_item_path_a = abs_item_path_a.as_ref();
        let abs_item_path_b = abs_item_path_b.as_ref();

        match self {
            Self::Name => {
                let file_name_a = abs_item_path_a.file_name();
                let file_name_b = abs_item_path_b.file_name();
                file_name_a.cmp(&file_name_b)
            },
            Self::ModTime => {
                let mtime_a = Util::mtime(abs_item_path_a);
                let mtime_b = Util::mtime(abs_item_path_b);
                mtime_a.cmp(&mtime_b)
            },
        }
    }
}

impl Default for SortBy {
    fn default() -> Self {
        Self::Name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs::File;
    use std::time::Duration;

    use tempfile::Builder;

    #[test]
    fn path_sort_cmp() {
        // Create temp directory.
        let temp = Builder::new().tempdir().expect("cannot create temp dir");
        let tp = temp.path();

        let fps = vec![
            tp.join("file_b"),
            tp.join("file_a"),
            tp.join("file_d"),
            tp.join("file_e"),
            tp.join("file_c"),
        ];

        for fp in &fps {
            // LEARN: Because we're iterating over a ref to a vector, the iter
            // vars are also refs.
            File::create(fp).expect(&format!(r#"unable to create file "{:?}""#, fp));
            std::thread::sleep(Duration::from_millis(10));
        }

        // Test sorting by mod time.
        let sort_order = SortBy::ModTime;

        for (o_i, o_val) in fps.iter().enumerate() {
            for (i_i, i_val) in fps.iter().enumerate() {
                assert_eq!(o_i.cmp(&i_i), sort_order.path_sort_cmp(o_val, i_val));
            }
        }

        // Test sorting by name.
        let sort_order = SortBy::Name;

        for o_val in fps.iter() {
            for i_val in fps.iter() {
                assert_eq!(
                    o_val.file_name().cmp(&i_val.file_name()),
                    sort_order.path_sort_cmp(o_val, i_val)
                );
            }
        }
    }
}
