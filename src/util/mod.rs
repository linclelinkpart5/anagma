pub mod file_walker;
pub mod number;

pub use number::Number;

use std::path::Path;
use std::path::PathBuf;
use std::path::Component;
use std::time::SystemTime;

/// Helpful utilities, meant to use used internally in the crate.
pub(crate) struct Util;

impl Util {
    /// Convenience method that gets the mod time of a path.
    /// Errors are coerced to `None`.
    pub fn mtime<P: AsRef<Path>>(abs_path: P) -> Option<SystemTime> {
        abs_path.as_ref().metadata().and_then(|m| m.modified()).ok()
    }

    /// Tests a string to see if it would be a valid item file name.
    pub fn _is_valid_item_name<S: AsRef<str>>(s: S) -> bool {
        let s = s.as_ref();
        let s_path = Path::new(s);
        let components: Vec<_> = s_path.components().collect();

        // If an item name does not have exactly one component, it is invalid.
        if components.len() != 1 {
            return false;
        }

        // The single component must be normal.
        match components[0] {
            Component::Normal(_) => {},
            _ => { return false; },
        }

        // Recreating the path from the component must match the original.
        // If not, the item name is invalid.
        let mut p = PathBuf::new();
        for c in components {
            p.push(c.as_os_str());
        }

        p.as_os_str() == s_path.as_os_str()
    }

    pub fn _separate_err<T, E>(results: Vec<Result<T, E>>) -> (Vec<T>, Vec<E>)
    where
        T: std::fmt::Debug,
        E: std::fmt::Debug,
    {
        let (values, errors): (Vec<_>, Vec<_>) = results.into_iter().partition(Result::is_ok);

        let values: Vec<_> = values.into_iter().map(Result::unwrap).collect();
        let errors: Vec<_> = errors.into_iter().map(Result::unwrap_err).collect();

        (values, errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs::File;
    use std::time::SystemTime;

    use tempfile::Builder;

    #[test]
    // NOTE: Using `SystemTime` is not guaranteed to be monotonic, so this test might be fragile.
    fn mtime() {
        // Create temp directory.
        let temp = Builder::new().suffix("test_mtime").tempdir().expect("unable to create temp directory");
        let tp = temp.path();

        let time_a = SystemTime::now();

        std::thread::sleep(std::time::Duration::from_millis(10));

        // Create a file to get the mtime of.
        let path = tp.join("file");
        File::create(&path).unwrap();

        std::thread::sleep(std::time::Duration::from_millis(10));

        let time_b = SystemTime::now();

        let file_time = Util::mtime(&path).unwrap();

        assert_eq!(time_a < file_time, true);
        assert_eq!(file_time < time_b, true);

        // Test getting time of nonexistent file.
        assert_eq!(None, Util::mtime(tp.join("DOES_NOT_EXIST")));
    }
}
