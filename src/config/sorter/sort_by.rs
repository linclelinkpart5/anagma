use std::path::Path;
use std::cmp::Ordering;

use serde::Deserialize;

use crate::util::Util;

fn name_cmp<P: AsRef<Path>>(abs_path_a: &P, abs_path_b: &P) -> Ordering {
    let file_name_a = abs_path_a.as_ref().file_name();
    let file_name_b = abs_path_b.as_ref().file_name();
    file_name_a.cmp(&file_name_b)
}

fn mtime_cmp<P: AsRef<Path>>(abs_path_a: &P, abs_path_b: &P) -> Ordering {
    let mtime_a = Util::mtime(abs_path_a.as_ref());
    let mtime_b = Util::mtime(abs_path_b.as_ref());
    mtime_a.cmp(&mtime_b)
}

/// Represents all criteria that can be used for sorting item files.
#[derive(Debug, Copy, Clone, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum SortBy {
    Name,
    ModTime,
}

impl SortBy {
    /// Compares two absolute item paths using this sorting criteria.
    pub fn cmp_paths<P>(&self, abs_path_a: &P, abs_path_b: &P) -> Ordering
    where
        P: AsRef<Path>,
    {
        let cmp_func = match self {
            Self::Name => name_cmp,
            Self::ModTime => mtime_cmp,
        };

        cmp_func(abs_path_a, abs_path_b)
    }

    /// Compares two `Result`s containing absolute item paths using this
    /// sorting criteria.
    pub fn cmp_path_results<P, E>(&self, res_a: &Result<P, E>, res_b: &Result<P, E>) -> Ordering
    where
        P: AsRef<Path>,
    {
        match (res_a, res_b) {
            (Ok(a), Ok(b)) => self.cmp_paths(a, b),
            (Err(_), Ok(_)) => Ordering::Less,
            (Ok(_), Err(_)) => Ordering::Greater,
            (Err(_), Err(_)) => Ordering::Equal,
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
    fn cmp_paths() {
        // Create temp directory.
        let temp = Builder::new().tempdir().unwrap();
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
            File::create(fp).unwrap();
            std::thread::sleep(Duration::from_millis(10));
        }

        // Test sorting by mod time.
        let sort_order = SortBy::ModTime;

        for (o_i, o_val) in fps.iter().enumerate() {
            for (i_i, i_val) in fps.iter().enumerate() {
                assert_eq!(o_i.cmp(&i_i), sort_order.cmp_paths(o_val, i_val));
            }
        }

        // Test sorting by name.
        let sort_order = SortBy::Name;

        for o_val in fps.iter() {
            for i_val in fps.iter() {
                assert_eq!(
                    o_val.file_name().cmp(&i_val.file_name()),
                    sort_order.cmp_paths(o_val, i_val)
                );
            }
        }
    }
}
