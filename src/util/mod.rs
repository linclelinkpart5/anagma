pub mod file_walker;
pub(crate) mod ooms;

pub use self::file_walker::FileWalker;

use std::fs::Metadata;
use std::io::Result as IoResult;
use std::path::{Path, Component};
use std::time::SystemTime;

use thiserror::Error;

#[derive(Debug, Clone, Copy, Error, PartialEq, Eq, Hash)]
pub enum InvalidNameKind {
    #[error("name does not have any path components")]
    EmptyName,
    #[error("name has more than one path component")]
    TooManyParts,
    #[error("name contains a non-normal path component")]
    NonNormalPart,
    #[error("name does not match normalized version of itself")]
    NotRoundTrip,
}

/// Helpful utilities, meant to use used internally in the crate.
pub(crate) struct Util;

impl Util {
    pub fn stat(path: &Path) -> IoResult<Metadata> {
        std::fs::metadata(path)
    }

    pub fn exists(path: &Path) -> IoResult<()> {
        Self::stat(path).map(|_| ())
    }

    pub fn is_file(path: &Path) -> IoResult<bool> {
        Self::stat(path).map(|m| m.is_file())
    }

    pub fn is_dir(path: &Path) -> IoResult<bool> {
        Self::stat(path).map(|m| m.is_dir())
    }

    /// Convenience method that gets the mod time of a path.
    /// Errors are coerced to `None`.
    pub fn mtime(abs_path: &Path) -> Option<SystemTime> {
        abs_path.metadata().and_then(|m| m.modified()).ok()
    }

    /// Tests a string to see if it would be a valid item file name.
    pub fn validate_item_name(name: &str) -> Result<(), InvalidNameKind> {
        // Re-create this name as a file path, and iterate over its components.
        let name_path = Path::new(name);
        let mut components = name_path.components();

        match (components.next(), components.next()) {
            (None, _) => { Err(InvalidNameKind::EmptyName) },
            (Some(_), Some(_)) => { Err(InvalidNameKind::TooManyParts) },
            (Some(Component::Normal(c)), None) => {
                if c != name { Err(InvalidNameKind::NotRoundTrip) }
                else { Ok(()) }
            },
            (Some(_), None) => { Err(InvalidNameKind::NonNormalPart) },
        }
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
    fn validate_item_name() {
        // Happy path.
        assert_eq!(
            Util::validate_item_name("name"),
            Ok(()),
        );
        assert_eq!(
            Util::validate_item_name(".name"),
            Ok(()),
        );
        assert_eq!(
            Util::validate_item_name("name."),
            Ok(()),
        );
        assert_eq!(
            Util::validate_item_name("name.ext"),
            Ok(()),
        );

        // Unhappy path.
        assert_eq!(
            Util::validate_item_name("."),
            Err(InvalidNameKind::NonNormalPart),
        );
        assert_eq!(
            Util::validate_item_name(".."),
            Err(InvalidNameKind::NonNormalPart),
        );
        assert_eq!(
            Util::validate_item_name("/"),
            Err(InvalidNameKind::NonNormalPart),
        );
        assert_eq!(
            Util::validate_item_name("/."),
            Err(InvalidNameKind::NonNormalPart),
        );
        assert_eq!(
            Util::validate_item_name("/.."),
            Err(InvalidNameKind::TooManyParts),
        );
        assert_eq!(
            Util::validate_item_name("./"),
            Err(InvalidNameKind::NonNormalPart),
        );
        assert_eq!(
            Util::validate_item_name("../"),
            Err(InvalidNameKind::NonNormalPart),
        );
        assert_eq!(
            Util::validate_item_name("/name"),
            Err(InvalidNameKind::TooManyParts),
        );
        assert_eq!(
            Util::validate_item_name("name/"),
            Err(InvalidNameKind::NotRoundTrip),
        );
        assert_eq!(
            Util::validate_item_name("./name"),
            Err(InvalidNameKind::TooManyParts),
        );
        assert_eq!(
            Util::validate_item_name("name/."),
            Err(InvalidNameKind::NotRoundTrip),
        );
        assert_eq!(
            Util::validate_item_name("../name"),
            Err(InvalidNameKind::TooManyParts),
        );
        assert_eq!(
            Util::validate_item_name("name/.."),
            Err(InvalidNameKind::TooManyParts),
        );
        assert_eq!(
            Util::validate_item_name("/name.ext"),
            Err(InvalidNameKind::TooManyParts),
        );
        assert_eq!(
            Util::validate_item_name("name.ext/"),
            Err(InvalidNameKind::NotRoundTrip),
        );
        assert_eq!(
            Util::validate_item_name("./name.ext"),
            Err(InvalidNameKind::TooManyParts),
        );
        assert_eq!(
            Util::validate_item_name("name.ext/."),
            Err(InvalidNameKind::NotRoundTrip),
        );
        assert_eq!(
            Util::validate_item_name("../name.ext"),
            Err(InvalidNameKind::TooManyParts),
        );
        assert_eq!(
            Util::validate_item_name("name.ext/.."),
            Err(InvalidNameKind::TooManyParts),
        );
    }
}
