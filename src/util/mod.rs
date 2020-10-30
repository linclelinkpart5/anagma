pub mod file_walker;
pub mod number;

// TODO: Just using these in preparation for refactoring, remove when these
//       are moved to this module.
pub use crate::config::selection::Selection;
pub use crate::config::selection::Matcher;
pub use crate::config::sorter::Sorter;
pub use crate::metadata::schema::SchemaFormat;

pub use number::Number;

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::path::Component;
use std::time::SystemTime;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum NameError {
    #[error("source name did not have any components")]
    NoComponents(PathBuf),
    #[error("source name had more than one component: {}", .0.display())]
    ManyComponents(PathBuf),
    #[error("source name contains a non-normal component: {}", .0.display())]
    NonNormalComponent(PathBuf),
    #[error("source name does not match normalized version of itself: {}", .0.display())]
    NotRoundTrip(PathBuf),
}

/// Helpful utilities, meant to use used internally in the crate.
pub(crate) struct Util;

impl Util {
    /// Convenience method that gets the mod time of a path.
    /// Errors are coerced to `None`.
    pub fn mtime(abs_path: &Path) -> Option<SystemTime> {
        abs_path.metadata().and_then(|m| m.modified()).ok()
    }

    /// Tests a string to see if it would be a valid item file name.
    pub fn validate_item_name<I: Into<PathBuf>>(name: I) -> Result<PathBuf, NameError> {
        // Re-create this name as a file path, and iterate over its components.
        let name = name.into();
        let mut components = name.components();

        match (components.next(), components.next()) {
            (None, _) => { Err(NameError::NoComponents(name)) },
            (Some(_), Some(_)) => { Err(NameError::ManyComponents(name)) },
            (Some(Component::Normal(c)), None) if c != &name => { Err(NameError::NotRoundTrip(name)) },
            (Some(Component::Normal(c)), None) => { Ok(name) },
            (Some(_), None) => { Err(NameError::NonNormalComponent(name)) },
        }
    }

    // TODO: CLEAN THIS UP, just leaving this here to avoid breaking tests.
    /// Tests a string to see if it would be a valid item file name.
    pub fn is_valid_item_name<P: AsRef<Path>>(name: &P) -> bool {
        // Re-create this name as a file path, and iterate over its components.
        let name = name.as_ref();
        let mut components = name.components();

        match (components.next(), components.next()) {
            // A valid path must have exactly one normal component.
            // It must also match the original name.
            (Some(Component::Normal(c)), None) => c == OsStr::new(name),
            _ => false,
        }
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
        let temp = Builder::new().suffix("mtime").tempdir().unwrap();
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
        assert_eq!(None, Util::mtime(&tp.join("DOES_NOT_EXIST")));
    }

    #[test]
    fn is_valid_item_name() {
        assert_eq!(Util::is_valid_item_name(&"name"), true);
        assert_eq!(Util::is_valid_item_name(&".name"), true);
        assert_eq!(Util::is_valid_item_name(&"name."), true);
        assert_eq!(Util::is_valid_item_name(&"name.ext"), true);

        assert_eq!(Util::is_valid_item_name(&"."), false);
        assert_eq!(Util::is_valid_item_name(&".."), false);
        assert_eq!(Util::is_valid_item_name(&"/"), false);
        assert_eq!(Util::is_valid_item_name(&"/."), false);
        assert_eq!(Util::is_valid_item_name(&"/.."), false);
        assert_eq!(Util::is_valid_item_name(&"./"), false);
        assert_eq!(Util::is_valid_item_name(&"../"), false);
        assert_eq!(Util::is_valid_item_name(&"/name"), false);
        assert_eq!(Util::is_valid_item_name(&"name/"), false);
        assert_eq!(Util::is_valid_item_name(&"./name"), false);
        assert_eq!(Util::is_valid_item_name(&"name/."), false);
        assert_eq!(Util::is_valid_item_name(&"../name"), false);
        assert_eq!(Util::is_valid_item_name(&"name/.."), false);
        assert_eq!(Util::is_valid_item_name(&"/name.ext"), false);
        assert_eq!(Util::is_valid_item_name(&"name.ext/"), false);
        assert_eq!(Util::is_valid_item_name(&"./name.ext"), false);
        assert_eq!(Util::is_valid_item_name(&"name.ext/."), false);
        assert_eq!(Util::is_valid_item_name(&"../name.ext"), false);
        assert_eq!(Util::is_valid_item_name(&"name.ext/.."), false);
    }
}
