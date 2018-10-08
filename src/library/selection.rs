//! Represents a method of determining whether a potential item path is to be included in metadata lookup.

use std::path::Path;

use globset::Glob;
use globset::GlobSet;
use globset::GlobSetBuilder;
use failure::Fail;
use failure::Error;
use failure::ResultExt;

#[derive(Clone, Eq, PartialEq, Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "invalid glob pattern: {}", _0)]
    InvalidSelectionPattern(String),
    #[fail(display = "cannot build selector")]
    CannotBuildSelector,
}

pub struct Selection(GlobSet);

impl Selection {
    pub fn from_patterns<II, S>(pattern_strs: II) -> Result<Self, Error>
    where
        II: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut builder = GlobSetBuilder::new();

        for pattern_str in pattern_strs.into_iter() {
            let pattern_str = pattern_str.as_ref();
            let pattern = Glob::new(&pattern_str).with_context(|_| ErrorKind::InvalidSelectionPattern(pattern_str.to_string()))?;
            builder.add(pattern);
        }

        let selection = builder.build().context(ErrorKind::CannotBuildSelector)?;

        Ok(Selection(selection))
    }

    pub fn is_match<P: AsRef<Path>>(&self, path: P) -> bool {
        self.0.is_match(path.as_ref())
    }

    pub fn any() -> Self {
        // NOTE: We assume that this is a universal pattern, and will not fail.
        Selection::from_patterns(&["*"]).unwrap()
    }

    pub fn empty() -> Self {
        Selection(GlobSet::empty())
    }
}

impl Default for Selection {
    fn default() -> Self {
        Selection::any()
    }
}

#[cfg(test)]
mod tests {
    use super::Selection;

    use std::path::Path;

    #[test]
    fn test_is_match() {
        let selection_a = Selection::from_patterns(&["*.a", "*.b"]).unwrap();
        let selection_b = Selection::from_patterns(&["*.b"]).unwrap();
        let selection_c = Selection::from_patterns(&["*.a", "*.c"]).unwrap();
        let selection_d = Selection::from_patterns(&["*"]).unwrap();

        assert_eq!(selection_a.is_match(Path::new("path.a")), true);
        assert_eq!(selection_a.is_match(Path::new("path.b")), true);
        assert_eq!(selection_a.is_match(Path::new("path.c")), false);
        assert_eq!(selection_a.is_match(Path::new("path.ab")), false);
        assert_eq!(selection_a.is_match(Path::new("path")), false);

        assert_eq!(selection_b.is_match(Path::new("path.a")), false);
        assert_eq!(selection_b.is_match(Path::new("path.b")), true);
        assert_eq!(selection_b.is_match(Path::new("path.c")), false);
        assert_eq!(selection_b.is_match(Path::new("path.ab")), false);
        assert_eq!(selection_b.is_match(Path::new("path")), false);

        assert_eq!(selection_c.is_match(Path::new("path.a")), true);
        assert_eq!(selection_c.is_match(Path::new("path.b")), false);
        assert_eq!(selection_c.is_match(Path::new("path.c")), true);
        assert_eq!(selection_c.is_match(Path::new("path.ab")), false);
        assert_eq!(selection_c.is_match(Path::new("path")), false);

        assert_eq!(selection_d.is_match(Path::new("path.a")), true);
        assert_eq!(selection_d.is_match(Path::new("path.b")), true);
        assert_eq!(selection_d.is_match(Path::new("path.c")), true);
        assert_eq!(selection_d.is_match(Path::new("path.ab")), true);
        assert_eq!(selection_d.is_match(Path::new("path")), true);
    }

    #[test]
    fn test_any() {
        let selection = Selection::any();

        assert_eq!(selection.is_match(Path::new("path")), true);
        assert_eq!(selection.is_match(Path::new("path.a")), true);
        assert_eq!(selection.is_match(Path::new("path.a.b.c")), true);
        assert_eq!(selection.is_match(Path::new("path.ab")), true);
        assert_eq!(selection.is_match(Path::new("")), true);
    }

    #[test]
    fn test_empty() {
        let selection = Selection::empty();

        assert_eq!(selection.is_match(Path::new("path")), false);
        assert_eq!(selection.is_match(Path::new("path.a")), false);
        assert_eq!(selection.is_match(Path::new("path.a.b.c")), false);
        assert_eq!(selection.is_match(Path::new("path.ab")), false);
        assert_eq!(selection.is_match(Path::new("")), false);
    }
}
