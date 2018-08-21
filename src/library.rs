//! A 'library' is an instance of a Taggu metadata hierarchy.

use std::path::Path;
use std::ffi::OsStr;
use std::fs::DirEntry;
use std::time::SystemTime;
use std::cmp::Ordering;

use regex::Regex;
use failure::Error;

#[derive(Debug, Clone)]
pub enum ItemSelection {
    Ext(String),
    Regex(Regex),
    IsFile,
    IsDir,
    And(Box<ItemSelection>, Box<ItemSelection>),
    Or(Box<ItemSelection>, Box<ItemSelection>),
    Xor(Box<ItemSelection>, Box<ItemSelection>),
    Not(Box<ItemSelection>),
    True,
    False,
}

impl ItemSelection {
    pub fn is_selected_path<P: AsRef<Path>>(&self, abs_item_path: P) -> bool {
        let abs_item_path = abs_item_path.as_ref();

        if !abs_item_path.exists() {
            return false
        }

        match *self {
            ItemSelection::Ext(ref e_ext) => abs_item_path.extension() == Some(&OsStr::new(e_ext)),
            ItemSelection::Regex(ref r_exp) => {
                abs_item_path
                    .file_name()
                    .and_then(|f| f.to_str())
                    .map_or(false, |f| r_exp.is_match(f))
            },
            ItemSelection::IsFile => abs_item_path.is_file(),
            ItemSelection::IsDir => abs_item_path.is_dir(),
            ItemSelection::And(ref sel_a, ref sel_b) => sel_a.is_selected_path(&abs_item_path)
                && sel_b.is_selected_path(&abs_item_path),
            ItemSelection::Or(ref sel_a, ref sel_b) => sel_a.is_selected_path(&abs_item_path)
                || sel_b.is_selected_path(&abs_item_path),
            ItemSelection::Xor(ref sel_a, ref sel_b) => sel_a.is_selected_path(&abs_item_path)
                ^ sel_b.is_selected_path(&abs_item_path),
            ItemSelection::Not(ref sel) => !sel.is_selected_path(&abs_item_path),
            ItemSelection::True => true,
            ItemSelection::False => false,
        }
    }

    pub fn selected_items_in_dir<P: AsRef<Path>>(&self, abs_dir_path: P) -> Result<Vec<DirEntry>, Error> {
        let abs_dir_path = abs_dir_path.as_ref();
        let dir_entries = abs_dir_path.read_dir()?;

        let mut sel_entries = vec![];

        for dir_entry in dir_entries {
            if let Ok(dir_entry) = dir_entry {
                if self.is_selected_path(dir_entry.path()) {
                    sel_entries.push(dir_entry);
                }
            } else {
                // TODO: Figure out what to do here.
            }
        }

        Ok(sel_entries)
    }

    // TODO: Create macros/functions to help with selection creation.
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ItemSortOrdering {
    Name,
    ModTime,
}

impl ItemSortOrdering {
    pub fn path_sort_cmp<P: AsRef<Path>>(&self, abs_item_path_a: P, abs_item_path_b: P) -> Ordering {
        let abs_item_path_a = abs_item_path_a.as_ref();
        let abs_item_path_b = abs_item_path_b.as_ref();

        match *self {
            ItemSortOrdering::Name => abs_item_path_a.file_name().cmp(&abs_item_path_b.file_name()),
            ItemSortOrdering::ModTime => ItemSortOrdering::get_mtime(abs_item_path_a).cmp(&ItemSortOrdering::get_mtime(abs_item_path_b)),
        }
    }

    fn get_mtime<P: AsRef<Path>>(abs_path: P) -> Option<SystemTime> {
        abs_path.as_ref().metadata().and_then(|m| m.modified()).ok()
    }
}

#[cfg(test)]
mod tests {
    extern crate tempdir;

    use std::path::PathBuf;
    use std::fs::DirBuilder;
    use std::fs::File;

    use self::tempdir::TempDir;
    use regex::Regex;

    use super::ItemSelection;

    #[test]
    fn test_is_selected_path() {
        // Create temp directory.
        let temp = TempDir::new("test_is_selected_path").unwrap();
        let tp = temp.path();

        // Generate desired file and dir paths.
        let mut paths_and_flags: Vec<(PathBuf, bool)> = vec![];

        let exts = vec!["flac", "ogg",];
        let suffixes = vec!["_a", "_b", "_aa",];

        for suffix in &suffixes {
            let f_path = tp.join(format!("file{}", suffix));
            paths_and_flags.push((f_path, false));

            let d_path = tp.join(format!("dir{}", suffix));
            paths_and_flags.push((d_path, true));

            for ext in &exts {
                let f_path = tp.join(format!("file{}.{}", suffix, ext));
                paths_and_flags.push((f_path, false));

                let d_path = tp.join(format!("dir{}.{}", suffix, ext));
                paths_and_flags.push((d_path, true));
            }
        }

        // Create the files and dirs.
        let db = DirBuilder::new();
        for &(ref path, is_dir) in &paths_and_flags {
            if is_dir {
                db.create(path).unwrap();
            } else {
                File::create(path).unwrap();
            }
        }

        // Test cases and indices of paths that should pass.
        let selections_and_true_indices = vec![
            (ItemSelection::IsFile, vec![0_usize, 2, 4, 6, 8, 10, 12, 14, 16]),
            (ItemSelection::IsDir, vec![1, 3, 5, 7, 9, 11, 13, 15, 17]),
            (ItemSelection::Ext("flac".to_string()), vec![2, 3, 8, 9, 14, 15]),
            (ItemSelection::Ext("ogg".to_string()), vec![4, 5, 10, 11, 16, 17]),
            (ItemSelection::Regex(Regex::new(r".*_a\..*").unwrap()), vec![2, 3, 4, 5]),
            (ItemSelection::And(
                Box::new(ItemSelection::IsFile),
                Box::new(ItemSelection::Ext("ogg".to_string())),
            ), vec![4, 10, 16]),
            (ItemSelection::Or(
                Box::new(ItemSelection::Ext("ogg".to_string())),
                Box::new(ItemSelection::Ext("flac".to_string())),
            ), vec![2, 3, 4, 5, 8, 9, 10, 11, 14, 15, 16, 17]),
            (ItemSelection::Or(
                Box::new(ItemSelection::IsDir),
                Box::new(ItemSelection::And(
                    Box::new(ItemSelection::IsFile),
                    Box::new(ItemSelection::Ext("flac".to_string())),
                )),
            ), vec![1, 2, 3, 5, 7, 8, 9, 11, 13, 14, 15, 17]),
            (ItemSelection::Xor(
                Box::new(ItemSelection::IsFile),
                Box::new(ItemSelection::Regex(Regex::new(r".*_a\..*").unwrap())),
            ), vec![0, 3, 5, 6, 8, 10, 12, 14, 16]),
            (ItemSelection::Not(
                Box::new(ItemSelection::IsFile),
            ), vec![1, 3, 5, 7, 9, 11, 13, 15, 17]),
            (ItemSelection::Not(
                Box::new(ItemSelection::Ext("flac".to_string())),
            ), vec![0, 1, 4, 5, 6, 7, 10, 11, 12, 13, 16, 17]),
            (ItemSelection::True, (0..18).collect()),
            (ItemSelection::False, vec![]),
        ];

        // Run the tests.
        for (selection, true_indices) in selections_and_true_indices {
            for (index, &(ref abs_path, _)) in paths_and_flags.iter().enumerate() {
                let expected = true_indices.contains(&index);
                let produced = selection.is_selected_path(&abs_path);
                assert_eq!(expected, produced);
            }
        }
    }
}
