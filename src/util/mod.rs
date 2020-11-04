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
    NoComponents,
    #[error("source name had more than one component: {0}")]
    ManyComponents(String),
    #[error("source name contains a non-normal component: {0}")]
    NonNormalComponent(String),
    #[error("source name does not match normalized version of itself: {0}")]
    NotRoundTrip(String),
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
    pub fn validate_item_name(name: String) -> Result<String, NameError> {
        // Re-create this name as a file path, and iterate over its components.
        let name_path = Path::new(&name);
        let mut components = name_path.components();

        match (components.next(), components.next()) {
            (None, _) => { Err(NameError::NoComponents) },
            (Some(_), Some(_)) => { Err(NameError::ManyComponents(name)) },
            (Some(Component::Normal(c)), None) => {
                if c != OsStr::new(&name) { Err(NameError::NotRoundTrip(name)) }
                else { Ok(name) }
            },
            (Some(_), None) => { Err(NameError::NonNormalComponent(name)) },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs::File;
    use std::time::SystemTime;

    use tempfile::Builder;

    use str_macro::str;

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
    fn validate_item_name() {
        assert_eq!(Util::validate_item_name(str!("name")).unwrap(), str!("name"));
        assert_eq!(Util::validate_item_name(str!(".name")).unwrap(), str!(".name"));
        assert_eq!(Util::validate_item_name(str!("name.")).unwrap(), str!("name."));
        assert_eq!(Util::validate_item_name(str!("name.ext")).unwrap(), str!("name.ext"));

        assert!(matches!(
            Util::validate_item_name(str!(".")),
            Err(NameError::NonNormalComponent(..))
        ));
        assert!(matches!(
            Util::validate_item_name(str!("..")),
            Err(NameError::NonNormalComponent(..))
        ));
        assert!(matches!(
            Util::validate_item_name(str!("/")),
            Err(NameError::NonNormalComponent(..))
        ));
        assert!(matches!(
            Util::validate_item_name(str!("/.")),
            Err(NameError::NonNormalComponent(..))
        ));
        assert!(matches!(
            Util::validate_item_name(str!("/..")),
            Err(NameError::ManyComponents(..))
        ));
        assert!(matches!(
            Util::validate_item_name(str!("./")),
            Err(NameError::NonNormalComponent(..))
        ));
        assert!(matches!(
            Util::validate_item_name(str!("../")),
            Err(NameError::NonNormalComponent(..))
        ));
        assert!(matches!(
            Util::validate_item_name(str!("/name")),
            Err(NameError::ManyComponents(..))
        ));
        assert!(matches!(
            Util::validate_item_name(str!("name/")),
            Err(NameError::NotRoundTrip(..))
        ));
        assert!(matches!(
            Util::validate_item_name(str!("./name")),
            Err(NameError::ManyComponents(..))
        ));
        assert!(matches!(
            Util::validate_item_name(str!("name/.")),
            Err(NameError::NotRoundTrip(..))
        ));
        assert!(matches!(
            Util::validate_item_name(str!("../name")),
            Err(NameError::ManyComponents(..))
        ));
        assert!(matches!(
            Util::validate_item_name(str!("name/..")),
            Err(NameError::ManyComponents(..))
        ));
        assert!(matches!(
            Util::validate_item_name(str!("/name.ext")),
            Err(NameError::ManyComponents(..))
        ));
        assert!(matches!(
            Util::validate_item_name(str!("name.ext/")),
            Err(NameError::NotRoundTrip(..))
        ));
        assert!(matches!(
            Util::validate_item_name(str!("./name.ext")),
            Err(NameError::ManyComponents(..))
        ));
        assert!(matches!(
            Util::validate_item_name(str!("name.ext/.")),
            Err(NameError::NotRoundTrip(..))
        ));
        assert!(matches!(
            Util::validate_item_name(str!("../name.ext")),
            Err(NameError::ManyComponents(..))
        ));
        assert!(matches!(
            Util::validate_item_name(str!("name.ext/..")),
            Err(NameError::ManyComponents(..))
        ));
    }
}
