//! Represents a method of determining whether a potential item path is to be included in metadata lookup.

use std::path::Path;
use std::hash::Hash;
use std::hash::Hasher;

use globset::Glob;
use globset::GlobSet;
use globset::GlobSetBuilder;
use globset::Error as GlobError;
use serde::Deserialize;
use serde::de::Deserializer;

#[derive(Debug)]
pub enum Error {
    InvalidPattern(GlobError),
    CannotBuildSelector(GlobError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::InvalidPattern(ref err) => write!(f, "invalid pattern: {}", err),
            Error::CannotBuildSelector(ref err) => write!(f, "cannot build selector: {}", err),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Error::InvalidPattern(ref err) => Some(err),
            Error::CannotBuildSelector(ref err) => Some(err),
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

        let matcher = builder.build().map_err(Error::CannotBuildSelector)?;

        // Sort and dedupe the patterns.
        cached_patterns.sort_by(|pa, pb| pa.glob().cmp(pb.glob()));
        cached_patterns.dedup();

        Ok(Matcher(matcher, cached_patterns))
    }

    pub fn is_match<P: AsRef<Path>>(&self, path: P) -> bool {
        self.0.is_match(path.as_ref())
    }

    pub fn any() -> Self {
        // NOTE: We assume that this is a universal pattern, and will not fail.
        Matcher::from_patterns(&["*"]).unwrap()
    }

    pub fn empty() -> Self {
        Matcher(GlobSet::empty(), vec![])
    }
}

impl Hash for Matcher {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.1.hash(state)
    }
}

impl PartialEq for Matcher {
    fn eq(&self, other: &Self) -> bool {
        self.1 == other.1
    }
}

impl Eq for Matcher {}

#[cfg(test)]
mod tests {
    use super::Matcher;
    use super::Error;

    use std::path::Path;

    use serde_yaml;

    #[test]
    fn test_deserialization() {
        let text = "'*.flac'";
        let matcher: Matcher = serde_yaml::from_str(&text).unwrap();

        assert!(matcher.is_match("music.flac"));
        assert!(!matcher.is_match("music.mp3"));
        assert!(!matcher.is_match("photo.png"));

        let text = "- '*.flac'\n- '*.mp3'";
        let matcher: Matcher = serde_yaml::from_str(&text).unwrap();

        assert!(matcher.is_match("music.flac"));
        assert!(matcher.is_match("music.mp3"));
        assert!(!matcher.is_match("photo.png"));
    }

    #[test]
    fn test_from_patterns() {
        let passing_inputs = vec![
            Matcher::from_patterns(&["*"]),
            Matcher::from_patterns(&["*.a", "*.b"]),
            Matcher::from_patterns(&["?.a", "?.b"]),
            Matcher::from_patterns(&["*.a"]),
            Matcher::from_patterns(&["**"]),
            Matcher::from_patterns(&["a/**/b"]),
            Matcher::from_patterns(&[""; 0]),
            Matcher::from_patterns(&[""]),
            Matcher::from_patterns(&["[a-z]*.a"]),
            Matcher::from_patterns(&["**", "[a-z]*.a"]),
            Matcher::from_patterns(&["[!abc]"]),
            Matcher::from_patterns(&["[*]"]),
            Matcher::from_patterns(&["[?]"]),
            Matcher::from_patterns(&["{*.a,*.b,*.c}"]),
        ];

        for input in passing_inputs {
            let expected = true;
            let produced = input.is_ok();
            assert_eq!(expected, produced);
        }

        let failing_inputs = vec![
            // Invalid double star
            Matcher::from_patterns(&["a**b"]),

            // Unclosed character class
            Matcher::from_patterns(&["[abc"]),

            // Malformed character range
            Matcher::from_patterns(&["[z-a]"]),

            // Unclosed alternates
            Matcher::from_patterns(&["{*.a,*.b,*.c"]),

            // Unopened alternates
            // Matcher::from_patterns(&["*.a,*.b,*.c}"]),

            // Nested alternates
            Matcher::from_patterns(&["{*.a,{*.b,*.c}}"]),

            // Dangling escape
            // Matcher::from_patterns(&["*.a\""]),
        ];

        for input in failing_inputs {
            match input.unwrap_err() {
                Error::InvalidPattern(_) => {},
                _ => { panic!(); },
            }
        }
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
        assert_eq!(matcher.is_match(Path::new("")), true);
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
