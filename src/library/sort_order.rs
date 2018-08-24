use std::path::Path;
use std::cmp::Ordering;
use std::time::SystemTime;

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
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

#[cfg(test)]
mod tests {
    extern crate tempdir;

    use std::path::PathBuf;
    use std::fs::DirBuilder;
    use std::fs::File;
    use std::thread;
    use std::time::Duration;
    use std::time::SystemTime;

    use self::tempdir::TempDir;
    use regex::Regex;

    use super::SortOrder;

    #[test]
    // NOTE: Using `SystemTime` is not guaranteed to be monotonic, so this test might be fragile.
    fn test_get_mtime() {
        // Create temp directory.
        let temp = TempDir::new("test_get_mtime").unwrap();
        let tp = temp.path();

        let time_a = SystemTime::now();

        thread::sleep(Duration::from_millis(10));

        // Create a file to get the mtime of.
        let path = tp.join("file");
        File::create(&path).unwrap();

        thread::sleep(Duration::from_millis(10));

        let time_b = SystemTime::now();

        let file_time = SortOrder::get_mtime(&path).unwrap();

        assert!(time_a < file_time);
        assert!(file_time < time_b);

        // Test getting time of nonexistent file.
        assert_eq!(None, SortOrder::get_mtime(tp.join("DOES_NOT_EXIST")));
    }
}
