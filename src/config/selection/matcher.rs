//! Represents a method of determining whether a potential item path is to be included in metadata lookup.

use std::path::Path;

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

impl OneOrManyPatterns {
    fn into_matcher(self) -> Result<Matcher, Error> {
        match self {
            OneOrManyPatterns::One(p) => {
                Matcher::from_patterns(&[p])
            },
            OneOrManyPatterns::Many(ps) => {
                Matcher::from_patterns(&ps)
            },
        }
    }
}

/// A filter for file paths, used to determine if a path is to be considered a metadata-containing item.
#[derive(Debug)]
pub struct Matcher(GlobSet, Vec<Glob>);

impl<'de> Deserialize<'de> for Matcher {
    fn deserialize<D>(deserializer: D) -> Result<Matcher, D::Error>
    where D: Deserializer<'de> {
        use serde::de::Error;
        let oom_patterns = OneOrManyPatterns::deserialize(deserializer).map_err(Error::custom)?;
        let matcher = oom_patterns.into_matcher().map_err(Error::custom)?;
        Ok(matcher)
    }
}

impl Matcher {
    pub fn from_patterns<II, S>(pattern_strs: II) -> Result<Self, Error>
    where
        II: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut builder = GlobSetBuilder::new();
        let mut cached_patterns = vec![];

        for pattern_str in pattern_strs.into_iter() {
            let pattern_str = pattern_str.as_ref();
            let pattern = Glob::new(&pattern_str).map_err(Error::InvalidPattern)?;
            builder.add(pattern.clone());
            cached_patterns.push(pattern);
        }

        let matcher = builder.build().map_err(Error::BuildFailure)?;

        // Sort and dedupe the patterns.
        cached_patterns.sort_by(|pa, pb| pa.glob().cmp(pb.glob()));
        cached_patterns.dedup();

        Ok(Matcher(matcher, cached_patterns))
    }

    pub fn is_match<P: AsRef<Path>>(&self, path: P) -> bool {
        // LEARN: Matching on the file name explicitly is needed for patterns such as "self*".
        path.as_ref().file_name().map(|f| self.0.is_match(f)).unwrap_or(false)
        // self.0.is_match(path.as_ref())
    }

    pub fn any() -> Self {
        // NOTE: We assume that this is a universal pattern, and will not fail.
        Matcher::from_patterns(&["*"]).unwrap()
    }

    pub fn empty() -> Self {
        Matcher(GlobSet::empty(), vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::Matcher;
    // use super::Error;

    use std::path::Path;

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
    fn test_from_patterns() {
        // Positive test cases.
        assert!(Matcher::from_patterns(&["*"]).is_ok());
        assert!(Matcher::from_patterns(&["*.a", "*.b"]).is_ok());
        assert!(Matcher::from_patterns(&["?.a", "?.b"]).is_ok());
        assert!(Matcher::from_patterns(&["*.a"]).is_ok());
        assert!(Matcher::from_patterns(&["**"]).is_ok());
        assert!(Matcher::from_patterns(&["a/**/b"]).is_ok());
        assert!(Matcher::from_patterns(&[""; 0]).is_ok());
        assert!(Matcher::from_patterns(&[""]).is_ok());
        assert!(Matcher::from_patterns(&["[a-z]*.a"]).is_ok());
        assert!(Matcher::from_patterns(&["**", "[a-z]*.a"]).is_ok());
        assert!(Matcher::from_patterns(&["[!abc]"]).is_ok());
        assert!(Matcher::from_patterns(&["[*]"]).is_ok());
        assert!(Matcher::from_patterns(&["[?]"]).is_ok());
        assert!(Matcher::from_patterns(&["{*.a,*.b,*.c}"]).is_ok());

        // Negative test cases.
        // Invalid double star.
        // assert!(Matcher::from_patterns(&["a**b"]).is_err());
        // Unclosed character class.
        assert!(Matcher::from_patterns(&["[abc"]).is_err());
        // Malformed character range.
        assert!(Matcher::from_patterns(&["[z-a]"]).is_err());
        // Unclosed alternates.
        assert!(Matcher::from_patterns(&["{*.a,*.b,*.c"]).is_err());
        // Unopened alternates.
        // assert!(Matcher::from_patterns(&["*.a,*.b,*.c}"]).is_err());
        // Nested alternates.
        assert!(Matcher::from_patterns(&["{*.a,{*.b,*.c}}"]).is_err());
        // Dangling escape.
        assert!(Matcher::from_patterns(&["*.a\\"]).is_err());
    }

    #[test]
    fn test_is_match() {
        let matcher_a = Matcher::from_patterns(&["*.a", "*.b"]).unwrap();
        let matcher_b = Matcher::from_patterns(&["*.b"]).unwrap();
        let matcher_c = Matcher::from_patterns(&["*.a", "*.c"]).unwrap();
        let matcher_d = Matcher::from_patterns(&["*"]).unwrap();

        assert_eq!(matcher_a.is_match(Path::new("path.a")), true);
        assert_eq!(matcher_a.is_match(Path::new("path.b")), true);
        assert_eq!(matcher_a.is_match(Path::new("path.c")), false);
        assert_eq!(matcher_a.is_match(Path::new("path.ab")), false);
        assert_eq!(matcher_a.is_match(Path::new("path")), false);

        assert_eq!(matcher_b.is_match(Path::new("path.a")), false);
        assert_eq!(matcher_b.is_match(Path::new("path.b")), true);
        assert_eq!(matcher_b.is_match(Path::new("path.c")), false);
        assert_eq!(matcher_b.is_match(Path::new("path.ab")), false);
        assert_eq!(matcher_b.is_match(Path::new("path")), false);

        assert_eq!(matcher_c.is_match(Path::new("path.a")), true);
        assert_eq!(matcher_c.is_match(Path::new("path.b")), false);
        assert_eq!(matcher_c.is_match(Path::new("path.c")), true);
        assert_eq!(matcher_c.is_match(Path::new("path.ab")), false);
        assert_eq!(matcher_c.is_match(Path::new("path")), false);

        assert_eq!(matcher_d.is_match(Path::new("path.a")), true);
        assert_eq!(matcher_d.is_match(Path::new("path.b")), true);
        assert_eq!(matcher_d.is_match(Path::new("path.c")), true);
        assert_eq!(matcher_d.is_match(Path::new("path.ab")), true);
        assert_eq!(matcher_d.is_match(Path::new("path")), true);
    }

    #[test]
    fn test_any() {
        let matcher = Matcher::any();

        assert_eq!(matcher.is_match(Path::new("path")), true);
        assert_eq!(matcher.is_match(Path::new("path.a")), true);
        assert_eq!(matcher.is_match(Path::new("path.a.b.c")), true);
        assert_eq!(matcher.is_match(Path::new("path.ab")), true);
        assert_eq!(matcher.is_match(Path::new("")), false);
    }

    #[test]
    fn test_empty() {
        let matcher = Matcher::empty();

        assert_eq!(matcher.is_match(Path::new("path")), false);
        assert_eq!(matcher.is_match(Path::new("path.a")), false);
        assert_eq!(matcher.is_match(Path::new("path.a.b.c")), false);
        assert_eq!(matcher.is_match(Path::new("path.ab")), false);
        assert_eq!(matcher.is_match(Path::new("")), false);
    }
}
