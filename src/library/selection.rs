use std::path::Path;
use std::ffi::OsStr;
use std::fs::DirEntry;

use regex::Regex;
use failure::Error;
use globset::Glob;
use globset::GlobSet;
use globset::GlobSetBuilder;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Selection {
    #[serde(rename = "ext")]
    Ext(String),

    #[serde(with = "serde_regex")]
    #[serde(rename = "regex")]
    Regex(Regex),

    #[serde(rename = "is_file")]
    IsFile,

    #[serde(rename = "is_dir")]
    IsDir,

    #[serde(rename = "and")]
    And(Box<Selection>, Box<Selection>),

    #[serde(rename = "or")]
    Or(Box<Selection>, Box<Selection>),

    #[serde(rename = "xor")]
    Xor(Box<Selection>, Box<Selection>),

    #[serde(rename = "not")]
    Not(Box<Selection>),

    #[serde(rename = "all")]
    All,

    #[serde(rename = "none")]
    None,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SimpleSelection(Vec<String>);

impl SimpleSelection {
    pub fn create_globset(&self) -> Result<GlobSet, Error> {
        let mut builder = GlobSetBuilder::new();

        for pattern in &self.0 {
            builder.add(Glob::new(&pattern)?);
        }

        Ok(builder.build()?)
    }
}

impl Selection {
    pub fn is_selected_path<P: AsRef<Path>>(&self, abs_item_path: P) -> bool {
        let abs_item_path = abs_item_path.as_ref();

        if !abs_item_path.exists() {
            return false
        }

        match *self {
            Selection::Ext(ref e_ext) => abs_item_path.extension() == Some(&OsStr::new(e_ext)),
            Selection::Regex(ref r_exp) => {
                abs_item_path
                    .file_name()
                    .and_then(|f| f.to_str())
                    .map_or(false, |f| r_exp.is_match(f))
            },
            Selection::IsFile => abs_item_path.is_file(),
            Selection::IsDir => abs_item_path.is_dir(),
            Selection::And(ref sel_a, ref sel_b) => sel_a.is_selected_path(&abs_item_path)
                && sel_b.is_selected_path(&abs_item_path),
            Selection::Or(ref sel_a, ref sel_b) => sel_a.is_selected_path(&abs_item_path)
                || sel_b.is_selected_path(&abs_item_path),
            Selection::Xor(ref sel_a, ref sel_b) => sel_a.is_selected_path(&abs_item_path)
                ^ sel_b.is_selected_path(&abs_item_path),
            Selection::Not(ref sel) => !sel.is_selected_path(&abs_item_path),
            Selection::All => true,
            Selection::None => false,
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

#[cfg(test)]
mod tests {
    extern crate tempdir;

    use std::path::PathBuf;
    use std::fs::DirBuilder;
    use std::fs::File;

    use self::tempdir::TempDir;
    use regex::Regex;

    use super::Selection;

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
            (Selection::IsFile, vec![0_usize, 2, 4, 6, 8, 10, 12, 14, 16]),
            (Selection::IsDir, vec![1, 3, 5, 7, 9, 11, 13, 15, 17]),
            (Selection::Ext("flac".to_string()), vec![2, 3, 8, 9, 14, 15]),
            (Selection::Ext("ogg".to_string()), vec![4, 5, 10, 11, 16, 17]),
            (Selection::Regex(Regex::new(r".*_a\..*").unwrap()), vec![2, 3, 4, 5]),
            (Selection::And(
                Box::new(Selection::IsFile),
                Box::new(Selection::Ext("ogg".to_string())),
            ), vec![4, 10, 16]),
            (Selection::Or(
                Box::new(Selection::Ext("ogg".to_string())),
                Box::new(Selection::Ext("flac".to_string())),
            ), vec![2, 3, 4, 5, 8, 9, 10, 11, 14, 15, 16, 17]),
            (Selection::Or(
                Box::new(Selection::IsDir),
                Box::new(Selection::And(
                    Box::new(Selection::IsFile),
                    Box::new(Selection::Ext("flac".to_string())),
                )),
            ), vec![1, 2, 3, 5, 7, 8, 9, 11, 13, 14, 15, 17]),
            (Selection::Xor(
                Box::new(Selection::IsFile),
                Box::new(Selection::Regex(Regex::new(r".*_a\..*").unwrap())),
            ), vec![0, 3, 5, 6, 8, 10, 12, 14, 16]),
            (Selection::Not(
                Box::new(Selection::IsFile),
            ), vec![1, 3, 5, 7, 9, 11, 13, 15, 17]),
            (Selection::Not(
                Box::new(Selection::Ext("flac".to_string())),
            ), vec![0, 1, 4, 5, 6, 7, 10, 11, 12, 13, 16, 17]),
            (Selection::All, (0..18).collect()),
            (Selection::None, vec![]),
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
