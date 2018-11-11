use std::path::Path;
use std::cmp::Ordering;
use std::time::SystemTime;

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
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

impl Default for SortOrder {
    fn default() -> Self {
        SortOrder::Name
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::time::SystemTime;

    use tempfile::Builder;

    use super::SortOrder;

    #[test]
    fn test_path_sort_cmp() {
        // Create temp directory.
        let temp = Builder::new().suffix("test_path_sort_cmp").tempdir().expect("unable to create temp directory");;
        let tp = temp.path();

        let fps = vec![
            tp.join("file_b"),
            tp.join("file_a"),
            tp.join("file_d"),
            tp.join("file_e"),
            tp.join("file_c"),
        ];

        for fp in &fps {
            // LEARN: Because we're iterating over a ref to a vector, the iter vars are also refs.
            File::create(fp).expect(&format!(r#"Unable to create file "{:?}""#, fp));
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        // Test sorting by mod time.
        let sort_order = SortOrder::ModTime;

        for (o_i, o_val) in fps.iter().enumerate() {
            for (i_i, i_val) in fps.iter().enumerate() {
                assert_eq!(o_i.cmp(&i_i), sort_order.path_sort_cmp(o_val, i_val));
            }
        }

        // Test sorting by name.
        let sort_order = SortOrder::Name;

        for o_val in fps.iter() {
            for i_val in fps.iter() {
                assert_eq!(o_val.file_name().cmp(&i_val.file_name()), sort_order.path_sort_cmp(o_val, i_val));
            }
        }
    }

    #[test]
    // NOTE: Using `SystemTime` is not guaranteed to be monotonic, so this test might be fragile.
    fn test_get_mtime() {
        // Create temp directory.
        let temp = Builder::new().suffix("test_get_mtime").tempdir().expect("unable to create temp directory");;
        let tp = temp.path();

        let time_a = SystemTime::now();

        std::thread::sleep(std::time::Duration::from_millis(10));

        // Create a file to get the mtime of.
        let path = tp.join("file");
        File::create(&path).unwrap();

        std::thread::sleep(std::time::Duration::from_millis(10));

        let time_b = SystemTime::now();

        let file_time = SortOrder::get_mtime(&path).unwrap();

        assert!(time_a < file_time);
        assert!(file_time < time_b);

        // Test getting time of nonexistent file.
        assert_eq!(None, SortOrder::get_mtime(tp.join("DOES_NOT_EXIST")));
    }
}
