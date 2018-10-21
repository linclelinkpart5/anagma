//! Represents a method of determining whether a potential item path is to be included in metadata lookup.

use std::fmt::Display;
use std::fmt::Result as FmtResult;
use std::fmt::Formatter;

use failure::Backtrace;
use failure::Context;
use failure::Fail;
use failure::ResultExt;

#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Fail, Hash)]
#[non_exhaustive]
pub enum ErrorKind {
    #[fail(display = "invalid pattern")]
    InvalidPattern,
    #[fail(display = "cannot build selector")]
    CannotBuildSelector,
}

impl Fail for Error {
    fn cause(&self) -> Option<&Fail> { self.inner.cause() }
    fn backtrace(&self) -> Option<&Backtrace> { self.inner.backtrace() }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult { Display::fmt(&self.inner, f) }
}

impl Error {
    pub fn kind(&self) -> &ErrorKind { self.inner.get_context() }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error { Error { inner: Context::new(kind) } }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Error { Error { inner: inner } }
}

use std::path::Path;
use std::path::PathBuf;

use globset::Glob;
use globset::Error as GlobError;
use globset::GlobSet;
use globset::GlobSetBuilder;
use serde::Deserialize;
use serde::de::Deserializer;

#[derive(Deserialize)]
#[serde(untagged)]
enum OneOrManyPatterns {
    One(String),
    Many(Vec<String>),
}

impl OneOrManyPatterns {
    fn into_selection(self) -> Result<Selection, Error> {
        match self {
            OneOrManyPatterns::One(p) => {
                Selection::from_patterns(&[p])
            },
            OneOrManyPatterns::Many(ps) => {
                Selection::from_patterns(&ps)
            },
        }
    }
}

/// A filter for file paths, used to determine if a path is to be considered a metadata-containing item.
#[derive(Debug)]
pub struct Selection(GlobSet);

impl<'de> Deserialize<'de> for Selection {
    fn deserialize<D>(deserializer: D) -> Result<Selection, D::Error>
    where D: Deserializer<'de> {
        use serde::de::Error;
        let oom_patterns = OneOrManyPatterns::deserialize(deserializer).map_err(Error::custom)?;
        let selection = oom_patterns.into_selection().map_err(Error::custom)?;
        Ok(selection)
    }
}

impl Selection {
    pub fn from_patterns<II, S>(pattern_strs: II) -> Result<Self, Error>
    where
        II: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut builder = GlobSetBuilder::new();

        for pattern_str in pattern_strs.into_iter() {
            let pattern_str = pattern_str.as_ref();
            let pattern = Glob::new(&pattern_str).context(ErrorKind::InvalidPattern)?;
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
    use super::ErrorKind;

    use std::path::Path;

    use serde_yaml;

    #[test]
    fn test_deserialization() {
        let text = "'*.flac'";
        let selection: Selection = serde_yaml::from_str(&text).unwrap();

        assert!(selection.is_match("music.flac"));
        assert!(!selection.is_match("music.mp3"));
        assert!(!selection.is_match("photo.png"));

        let text = "- '*.flac'\n- '*.mp3'";
        let selection: Selection = serde_yaml::from_str(&text).unwrap();

        assert!(selection.is_match("music.flac"));
        assert!(selection.is_match("music.mp3"));
        assert!(!selection.is_match("photo.png"));
    }

    #[test]
    fn test_from_patterns() {
        let passing_inputs = vec![
            Selection::from_patterns(&["*"]),
            Selection::from_patterns(&["*.a", "*.b"]),
            Selection::from_patterns(&["?.a", "?.b"]),
            Selection::from_patterns(&["*.a"]),
            Selection::from_patterns(&["**"]),
            Selection::from_patterns(&["a/**/b"]),
            Selection::from_patterns(&[""; 0]),
            Selection::from_patterns(&[""]),
            Selection::from_patterns(&["[a-z]*.a"]),
            Selection::from_patterns(&["**", "[a-z]*.a"]),
            Selection::from_patterns(&["[!abc]"]),
            Selection::from_patterns(&["[*]"]),
            Selection::from_patterns(&["[?]"]),
            Selection::from_patterns(&["{*.a,*.b,*.c}"]),
        ];

        for input in passing_inputs {
            let expected = true;
            let produced = input.is_ok();
            assert_eq!(expected, produced);
        }

        let failing_inputs = vec![
            // Invalid double star
            Selection::from_patterns(&["a**b"]),

            // Unclosed character class
            Selection::from_patterns(&["[abc"]),

            // Malformed character range
            Selection::from_patterns(&["[z-a]"]),

            // Unclosed alternates
            Selection::from_patterns(&["{*.a,*.b,*.c"]),

            // Unopened alternates
            // Selection::from_patterns(&["*.a,*.b,*.c}"]),

            // Nested alternates
            Selection::from_patterns(&["{*.a,{*.b,*.c}}"]),

            // Dangling escape
            // Selection::from_patterns(&["*.a\""]),
        ];

        for input in failing_inputs {
            match input.unwrap_err().kind() {
                ErrorKind::InvalidPattern => {},
                _ => { panic!(); },
            }
        }
    }

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
