//! Defines item file sorting order.

pub mod sort_by;

use std::cmp::Ordering;
use std::path::Path;

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

    pub fn sort_path_results<P, E>(&self, res_paths: &mut [Result<P, E>])
    where
        P: AsRef<Path>,
    {
        res_paths.sort_by(|res_a, res_b| {
            match (res_a, res_b) {
                (Ok(a), Ok(b)) => self.cmp_paths(a, b),

                // These should ensure that errors always get sorted to the front.
                (Err(_), Ok(_)) => Ordering::Less,
                (Ok(_), Err(_)) => Ordering::Greater,
                (Err(_), Err(_)) => Ordering::Equal,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rand::seq::SliceRandom;

    use crate::test_util::TestUtil;

    #[test]
    fn sort_paths() {
        let file_names = &["file_b", "file_e", "file_a", "file_c", "file_d"];
        let temp_dir = TestUtil::create_simple_dir("sort_paths", file_names);
        let temp_dir_path = temp_dir.path();

        let mut input = file_names
            .iter()
            .map(|n| temp_dir_path.join(n))
            .collect::<Vec<_>>();
        input.shuffle(&mut rand::thread_rng());

        // Sort by name, ascending.
        let expected = vec![
            temp_dir_path.join("file_a"),
            temp_dir_path.join("file_b"),
            temp_dir_path.join("file_c"),
            temp_dir_path.join("file_d"),
            temp_dir_path.join("file_e"),
        ];
        let sorter = Sorter {
            sort_by: SortBy::Name,
            sort_order: SortOrder::Ascending,
        };
        let mut produced = input.clone();
        sorter.sort_paths(&mut produced);
        assert_eq!(produced, expected);

        // Sort by name, descending.
        let expected = vec![
            temp_dir_path.join("file_e"),
            temp_dir_path.join("file_d"),
            temp_dir_path.join("file_c"),
            temp_dir_path.join("file_b"),
            temp_dir_path.join("file_a"),
        ];
        let sorter = Sorter {
            sort_by: SortBy::Name,
            sort_order: SortOrder::Descending,
        };
        let mut produced = input.clone();
        sorter.sort_paths(&mut produced);
        assert_eq!(produced, expected);

        // Sort by mod time, ascending.
        let expected = vec![
            temp_dir_path.join("file_b"),
            temp_dir_path.join("file_e"),
            temp_dir_path.join("file_a"),
            temp_dir_path.join("file_c"),
            temp_dir_path.join("file_d"),
        ];
        let sorter = Sorter {
            sort_by: SortBy::ModTime,
            sort_order: SortOrder::Ascending,
        };
        let mut produced = input.clone();
        sorter.sort_paths(&mut produced);
        assert_eq!(produced, expected);

        // Sort by mod time, descending.
        let expected = vec![
            temp_dir_path.join("file_d"),
            temp_dir_path.join("file_c"),
            temp_dir_path.join("file_a"),
            temp_dir_path.join("file_e"),
            temp_dir_path.join("file_b"),
        ];
        let sorter = Sorter {
            sort_by: SortBy::ModTime,
            sort_order: SortOrder::Descending,
        };
        let mut produced = input.clone();
        sorter.sort_paths(&mut produced);
        assert_eq!(produced, expected);
    }

    #[test]
    fn sort_path_results() {
        #[derive(Debug, Clone, Copy, PartialEq)]
        struct Error(u32);

        let file_names = &["file_b", "file_e", "file_a", "file_c", "file_d"];
        let temp_dir = TestUtil::create_simple_dir("sort_path_results", file_names);
        let temp_dir_path = temp_dir.path();

        let mut input = file_names
            .iter()
            .map(|n| temp_dir_path.join(n))
            .map(Result::Ok)
            .collect::<Vec<_>>();
        input.push(Err(Error(0)));
        input.push(Err(Error(0)));
        input.push(Err(Error(0)));
        input.shuffle(&mut rand::thread_rng());

        // Mutate the errors in input to have a deterministic relative order.
        let mut cycle = (1u32..=3).into_iter().cycle();
        for i in input.iter_mut() {
            match i {
                Ok(..) => {}
                Err(err) => {
                    err.0 = cycle.next().unwrap();
                }
            }
        }

        // Sort by name, ascending.
        let expected = vec![
            Err(Error(1)),
            Err(Error(2)),
            Err(Error(3)),
            Ok(temp_dir_path.join("file_a")),
            Ok(temp_dir_path.join("file_b")),
            Ok(temp_dir_path.join("file_c")),
            Ok(temp_dir_path.join("file_d")),
            Ok(temp_dir_path.join("file_e")),
        ];
        let sorter = Sorter {
            sort_by: SortBy::Name,
            sort_order: SortOrder::Ascending,
        };
        let mut produced = input.clone();
        sorter.sort_path_results(&mut produced);
        assert_eq!(produced, expected);

        // Sort by name, descending.
        let expected = vec![
            Err(Error(1)),
            Err(Error(2)),
            Err(Error(3)),
            Ok(temp_dir_path.join("file_e")),
            Ok(temp_dir_path.join("file_d")),
            Ok(temp_dir_path.join("file_c")),
            Ok(temp_dir_path.join("file_b")),
            Ok(temp_dir_path.join("file_a")),
        ];
        let sorter = Sorter {
            sort_by: SortBy::Name,
            sort_order: SortOrder::Descending,
        };
        let mut produced = input.clone();
        sorter.sort_path_results(&mut produced);
        assert_eq!(produced, expected);

        // Sort by mod time, ascending.
        let expected = vec![
            Err(Error(1)),
            Err(Error(2)),
            Err(Error(3)),
            Ok(temp_dir_path.join("file_b")),
            Ok(temp_dir_path.join("file_e")),
            Ok(temp_dir_path.join("file_a")),
            Ok(temp_dir_path.join("file_c")),
            Ok(temp_dir_path.join("file_d")),
        ];
        let sorter = Sorter {
            sort_by: SortBy::ModTime,
            sort_order: SortOrder::Ascending,
        };
        let mut produced = input.clone();
        sorter.sort_path_results(&mut produced);
        assert_eq!(produced, expected);

        // Sort by mod time, descending.
        let expected = vec![
            Err(Error(1)),
            Err(Error(2)),
            Err(Error(3)),
            Ok(temp_dir_path.join("file_d")),
            Ok(temp_dir_path.join("file_c")),
            Ok(temp_dir_path.join("file_a")),
            Ok(temp_dir_path.join("file_e")),
            Ok(temp_dir_path.join("file_b")),
        ];
        let sorter = Sorter {
            sort_by: SortBy::ModTime,
            sort_order: SortOrder::Descending,
        };
        let mut produced = input.clone();
        sorter.sort_path_results(&mut produced);
        assert_eq!(produced, expected);
    }
}
