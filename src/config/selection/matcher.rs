//! Represents a method of determining whether a potential item path is to be included in metadata lookup.

use std::path::Path;
use std::convert::TryFrom;
use std::convert::TryInto;

use globset::Glob;
use globset::GlobSet;
use globset::GlobSetBuilder;
use globset::Error as GlobError;
use serde::Deserialize;
use serde::de::Deserializer;

#[derive(Debug)]
pub enum Error {
    InvalidPattern(GlobError),
    BuildFailure(GlobError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::InvalidPattern(ref err) => write!(f, "invalid pattern: {}", err),
            Error::BuildFailure(ref err) => write!(f, "cannot build matcher: {}", err),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Error::InvalidPattern(ref err) => Some(err),
            Error::BuildFailure(ref err) => Some(err),
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum OneOrManyPatterns {
    One(String),
    Many(Vec<String>),
}

impl TryFrom<OneOrManyPatterns> for Matcher {
    type Error = Error;

    fn try_from(oom: OneOrManyPatterns) -> Result<Self, Self::Error> {
        match oom {
            OneOrManyPatterns::One(p) => Matcher::build(&[p]),
            OneOrManyPatterns::Many(ps) => Matcher::build(&ps),
        }
    }
}

/// A filter for file paths, used to determine if a path is to be considered a metadata-containing item.
#[derive(Debug)]
pub struct Matcher(GlobSet);

impl<'de> Deserialize<'de> for Matcher {
    fn deserialize<D>(deserializer: D) -> Result<Matcher, D::Error>
    where D: Deserializer<'de> {
        use serde::de::Error;
        let oom_patterns = OneOrManyPatterns::deserialize(deserializer).map_err(Error::custom)?;
        let matcher = oom_patterns.try_into().map_err(Error::custom)?;
        Ok(matcher)
    }
}

impl Matcher {
    pub fn build<II, S>(pattern_strs: II) -> Result<Self, Error>
    where
        II: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut builder = GlobSetBuilder::new();

        for pattern_str in pattern_strs.into_iter() {
            let pattern_str = pattern_str.as_ref();
            let pattern = Glob::new(&pattern_str).map_err(Error::InvalidPattern)?;
            builder.add(pattern);
        }

        let matcher = builder.build().map_err(Error::BuildFailure)?;

        Ok(Matcher(matcher))
    }

    pub fn is_match<P: AsRef<Path>>(&self, path: P) -> bool {
        // LEARN: Matching on the file name explicitly is needed for patterns such as "self*".
        path.as_ref().file_name().map(|f| self.0.is_match(f)).unwrap_or(false)
    }

    pub fn any() -> Self {
        // NOTE: We assume that this is a universal pattern, and will not fail.
        Matcher::build(&["*"]).unwrap()
    }

    pub fn empty() -> Self {
        Matcher(GlobSet::empty())
    }
}

#[cfg(test)]
mod tests {
    use super::Matcher;

    #[test]
    fn test_deserialization() {
        let text = "'*.flac'";
        let matcher: Matcher = serde_yaml::from_str(&text).unwrap();

        assert_eq!(matcher.is_match("music.flac"), true);
        assert_eq!(matcher.is_match("music.mp3"), false);
        assert_eq!(matcher.is_match("photo.png"), false);

        let text = "- '*.flac'\n- '*.mp3'";
        let matcher: Matcher = serde_yaml::from_str(&text).unwrap();

        assert_eq!(matcher.is_match("music.flac"), true);
        assert_eq!(matcher.is_match("music.mp3"), true);
        assert_eq!(matcher.is_match("photo.png"), false);
    }

    #[test]
    fn test_build() {
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
    fn test_is_match() {
        let matcher = Matcher::build(&["*.a", "*.b"]).unwrap();
        assert_eq!(matcher.is_match("path.a"), true);
        assert_eq!(matcher.is_match("path.b"), true);
        assert_eq!(matcher.is_match("path.c"), false);
        assert_eq!(matcher.is_match("path.ab"), false);
        assert_eq!(matcher.is_match("path"), false);

        let matcher = Matcher::build(&["*.b"]).unwrap();
        assert_eq!(matcher.is_match("path.a"), false);
        assert_eq!(matcher.is_match("path.b"), true);
        assert_eq!(matcher.is_match("path.c"), false);
        assert_eq!(matcher.is_match("path.ab"), false);
        assert_eq!(matcher.is_match("path"), false);

        let matcher = Matcher::build(&["*.a", "*.c"]).unwrap();
        assert_eq!(matcher.is_match("path.a"), true);
        assert_eq!(matcher.is_match("path.b"), false);
        assert_eq!(matcher.is_match("path.c"), true);
        assert_eq!(matcher.is_match("path.ab"), false);
        assert_eq!(matcher.is_match("path"), false);

        let matcher = Matcher::build(&["*"]).unwrap();
        assert_eq!(matcher.is_match("path.a"), true);
        assert_eq!(matcher.is_match("path.b"), true);
        assert_eq!(matcher.is_match("path.c"), true);
        assert_eq!(matcher.is_match("path.ab"), true);
        assert_eq!(matcher.is_match("path"), true);
    }

    #[test]
    fn test_any() {
        let matcher = Matcher::any();
        assert_eq!(matcher.is_match("path"), true);
        assert_eq!(matcher.is_match("path.a"), true);
        assert_eq!(matcher.is_match("path.a.b.c"), true);
        assert_eq!(matcher.is_match("path.ab"), true);
        assert_eq!(matcher.is_match(""), false);
    }

    #[test]
    fn test_empty() {
        let matcher = Matcher::empty();
        assert_eq!(matcher.is_match("path"), false);
        assert_eq!(matcher.is_match("path.a"), false);
        assert_eq!(matcher.is_match("path.a.b.c"), false);
        assert_eq!(matcher.is_match("path.ab"), false);
        assert_eq!(matcher.is_match(""), false);
    }
}
