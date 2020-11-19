use std::path::Path;
use std::convert::{TryFrom, TryInto};

use globset::Glob;
use globset::GlobSet;
use globset::GlobSetBuilder;
use globset::Error as GlobError;
use serde::Deserialize;
use thiserror::Error;

use crate::util::ooms::Ooms;

#[derive(Error, Debug)]
#[error("invalid pattern: {0}")]
pub struct PatternError(#[from] GlobError);

#[derive(Error, Debug)]
#[error("cannot build matcher: {0}")]
pub struct BuildError(#[from] GlobError);

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Pattern(#[from] PatternError),
    #[error("{0}")]
    Build(#[from] BuildError),
}

#[derive(Debug)]
pub(crate) struct MatcherBuilder(GlobSetBuilder);

impl MatcherBuilder {
    pub fn new() -> Self {
        Self(GlobSetBuilder::new())
    }

    pub fn add_pattern<S: AsRef<str>>(&mut self, pattern: &S) -> Result<(), PatternError> {
        self.add_glob(Glob::new(pattern.as_ref())?);
        Ok(())
    }

    pub fn add_glob(&mut self, glob: Glob) {
        self.0.add(glob);
    }

    pub fn build(self) -> Result<Matcher, BuildError> {
        Ok(Matcher(self.0.build()?))
    }
}

/// Filter for file paths that uses zero or more glob patterns to perform matching.
#[derive(Debug, Deserialize)]
#[serde(try_from = "MatcherRepr")]
pub struct Matcher(GlobSet);

impl Matcher {
    /// Attempts to build a matcher out of an iterable of string-likes.
    pub fn build<'a, I, S: 'a>(pattern_strs: I) -> Result<Self, Error>
    where
        I: IntoIterator<Item = &'a S>,
        S: AsRef<str>,
    {
        let mut builder = MatcherBuilder::new();

        for pattern_str in pattern_strs {
            builder.add_pattern(pattern_str)?;
        }

        Ok(builder.build()?)
    }

    /// Matches a path based on its file name. If the path does not have a file
    /// name (e.g. '/' on Unix systems), returns `false`.
    pub fn is_match<P: AsRef<Path>>(&self, path: &P) -> bool {
        // Matching on only file name is needed for patterns such as "self*".
        path.as_ref().file_name().map(|f| self.0.is_match(f)).unwrap_or(false)
    }

    /// Returns a matcher that matches any path that has a file name.
    pub fn any() -> Self {
        // Assume that this is a universal pattern, and will not fail.
        Self::build(&["*"]).unwrap()
    }

    /// Returns a matcher that matches no paths.
    pub fn empty() -> Self {
        Self(GlobSet::empty())
    }
}

#[derive(Debug, Deserialize)]
#[serde(try_from = "Ooms")]
pub(crate) enum MatcherRepr {
    Any,
    Empty,
    Custom(MatcherBuilder),
}

impl MatcherRepr {
    fn add_pattern<S: AsRef<str>>(&mut self, pattern: &S) -> Result<(), GlobError> {
        // Always verify that the pattern is valid.
        let glob = Glob::new(pattern.as_ref())?;
        self.add_glob(glob);
        Ok(())
    }

    fn add_glob(&mut self, glob: Glob) {
        match self {
            // No-op, all patterns are already included.
            Self::Any => {},

            // Redefine as a custom variant.
            Self::Empty => {
                let mut builder = MatcherBuilder::new();
                builder.add_glob(glob);

                *self = Self::Custom(builder);
            },

            // Add the pattern to the existing ones.
            Self::Custom(ref mut builder) => {
                builder.add_glob(glob);
            },
        };
    }
}

impl TryFrom<Ooms> for MatcherRepr {
    type Error = PatternError;

    fn try_from(value: Ooms) -> Result<Self, Self::Error> {
        let mut builder = MatcherBuilder::new();

        for pattern in value.iter() {
            builder.add_pattern(&pattern)?;
        }

        Ok(MatcherRepr::Custom(builder))
    }
}

impl TryFrom<MatcherRepr> for Matcher {
    type Error = BuildError;

    fn try_from(value: MatcherRepr) -> Result<Self, Self::Error> {
        match value {
            MatcherRepr::Any => Ok(Matcher::any()),
            MatcherRepr::Empty => Ok(Matcher::empty()),
            MatcherRepr::Custom(builder) => builder.build(),
        }
    }
}

impl TryFrom<Ooms> for Matcher {
    type Error = Error;

    fn try_from(value: Ooms) -> Result<Self, Self::Error> {
        let mr = TryInto::<MatcherRepr>::try_into(value)?;
        Ok(mr.try_into()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialization() {
        let text = "'*.flac'";
        let matcher: Matcher = serde_yaml::from_str(&text).unwrap();

        assert_eq!(matcher.is_match(&"music.flac"), true);
        assert_eq!(matcher.is_match(&"music.mp3"), false);
        assert_eq!(matcher.is_match(&"photo.png"), false);

        let text = "- '*.flac'\n- '*.mp3'";
        let matcher: Matcher = serde_yaml::from_str(&text).unwrap();

        assert_eq!(matcher.is_match(&"music.flac"), true);
        assert_eq!(matcher.is_match(&"music.mp3"), true);
        assert_eq!(matcher.is_match(&"photo.png"), false);
    }

    #[test]
    fn build() {
        // Positive test cases.
        assert!(Matcher::build(&["*"]).is_ok());
        assert!(Matcher::build(&["*.a", "*.b"]).is_ok());
        assert!(Matcher::build(&["?.a", "?.b"]).is_ok());
        assert!(Matcher::build(&["*.a"]).is_ok());
        assert!(Matcher::build(&["**"]).is_ok());
        assert!(Matcher::build(&["a/**/b"]).is_ok());
        assert!(Matcher::build(&[""; 0]).is_ok());
        assert!(Matcher::build(&[""]).is_ok());
        assert!(Matcher::build(&["[a-z]*.a"]).is_ok());
        assert!(Matcher::build(&["**", "[a-z]*.a"]).is_ok());
        assert!(Matcher::build(&["[!abc]"]).is_ok());
        assert!(Matcher::build(&["[*]"]).is_ok());
        assert!(Matcher::build(&["[?]"]).is_ok());
        assert!(Matcher::build(&["{*.a,*.b,*.c}"]).is_ok());

        // Negative test cases.
        // Invalid double star.
        // assert!(Matcher::build(&["a**b"]).is_err());
        // Unclosed character class.
        assert!(Matcher::build(&["[abc"]).is_err());
        // Malformed character range.
        assert!(Matcher::build(&["[z-a]"]).is_err());
        // Unclosed alternates.
        assert!(Matcher::build(&["{*.a,*.b,*.c"]).is_err());
        // Unopened alternates.
        // assert!(Matcher::build(&["*.a,*.b,*.c}"]).is_err());
        // Nested alternates.
        assert!(Matcher::build(&["{*.a,{*.b,*.c}}"]).is_err());
        // Dangling escape.
        assert!(Matcher::build(&["*.a\\"]).is_err());
    }

    #[test]
    fn is_match() {
        let matcher = Matcher::build(&["*.a", "*.b"]).unwrap();
        assert_eq!(matcher.is_match(&"path.a"), true);
        assert_eq!(matcher.is_match(&"path.b"), true);
        assert_eq!(matcher.is_match(&"path.c"), false);
        assert_eq!(matcher.is_match(&"path.ab"), false);
        assert_eq!(matcher.is_match(&"path"), false);
        assert_eq!(matcher.is_match(&"extra/path.a"), true);
        assert_eq!(matcher.is_match(&"extra/path.b"), true);
        assert_eq!(matcher.is_match(&"extra/path.c"), false);
        assert_eq!(matcher.is_match(&"/"), false);
        assert_eq!(matcher.is_match(&""), false);

        let matcher = Matcher::build(&["*.b"]).unwrap();
        assert_eq!(matcher.is_match(&"path.a"), false);
        assert_eq!(matcher.is_match(&"path.b"), true);
        assert_eq!(matcher.is_match(&"path.c"), false);
        assert_eq!(matcher.is_match(&"path.ab"), false);
        assert_eq!(matcher.is_match(&"path"), false);
        assert_eq!(matcher.is_match(&"extra/path.a"), false);
        assert_eq!(matcher.is_match(&"extra/path.b"), true);
        assert_eq!(matcher.is_match(&"extra/path.c"), false);
        assert_eq!(matcher.is_match(&"/"), false);
        assert_eq!(matcher.is_match(&""), false);

        let matcher = Matcher::build(&["*.a", "*.c"]).unwrap();
        assert_eq!(matcher.is_match(&"path.a"), true);
        assert_eq!(matcher.is_match(&"path.b"), false);
        assert_eq!(matcher.is_match(&"path.c"), true);
        assert_eq!(matcher.is_match(&"path.ab"), false);
        assert_eq!(matcher.is_match(&"path"), false);
        assert_eq!(matcher.is_match(&"extra/path.a"), true);
        assert_eq!(matcher.is_match(&"extra/path.b"), false);
        assert_eq!(matcher.is_match(&"extra/path.c"), true);
        assert_eq!(matcher.is_match(&"/"), false);
        assert_eq!(matcher.is_match(&""), false);

        let matcher = Matcher::build(&["*"]).unwrap();
        assert_eq!(matcher.is_match(&"path.a"), true);
        assert_eq!(matcher.is_match(&"path.b"), true);
        assert_eq!(matcher.is_match(&"path.c"), true);
        assert_eq!(matcher.is_match(&"path.ab"), true);
        assert_eq!(matcher.is_match(&"path"), true);
        assert_eq!(matcher.is_match(&"extra/path.a"), true);
        assert_eq!(matcher.is_match(&"extra/path.b"), true);
        assert_eq!(matcher.is_match(&"extra/path.c"), true);
        assert_eq!(matcher.is_match(&"/"), false);
        assert_eq!(matcher.is_match(&""), false);

        let matcher = Matcher::build(&[] as &[&str]).unwrap();
        assert_eq!(matcher.is_match(&"path.a"), false);
        assert_eq!(matcher.is_match(&"path.b"), false);
        assert_eq!(matcher.is_match(&"path.c"), false);
        assert_eq!(matcher.is_match(&"path.ab"), false);
        assert_eq!(matcher.is_match(&"path"), false);
        assert_eq!(matcher.is_match(&"extra/path.a"), false);
        assert_eq!(matcher.is_match(&"extra/path.b"), false);
        assert_eq!(matcher.is_match(&"extra/path.c"), false);
        assert_eq!(matcher.is_match(&"/"), false);
        assert_eq!(matcher.is_match(&""), false);
    }

    #[test]
    fn any() {
        let matcher = Matcher::any();
        assert_eq!(matcher.is_match(&"path"), true);
        assert_eq!(matcher.is_match(&"path.a"), true);
        assert_eq!(matcher.is_match(&"path.a.b.c"), true);
        assert_eq!(matcher.is_match(&"path.ab"), true);
        assert_eq!(matcher.is_match(&"/extra/path.a"), true);
        assert_eq!(matcher.is_match(&"extra/path.a"), true);
        assert_eq!(matcher.is_match(&"/"), false);
        assert_eq!(matcher.is_match(&""), false);
    }

    #[test]
    fn empty() {
        let matcher = Matcher::empty();
        assert_eq!(matcher.is_match(&"path"), false);
        assert_eq!(matcher.is_match(&"path.a"), false);
        assert_eq!(matcher.is_match(&"path.a.b.c"), false);
        assert_eq!(matcher.is_match(&"path.ab"), false);
        assert_eq!(matcher.is_match(&"/extra/path.a"), false);
        assert_eq!(matcher.is_match(&"extra/path.a"), false);
        assert_eq!(matcher.is_match(&"/"), false);
        assert_eq!(matcher.is_match(&""), false);
    }
}
