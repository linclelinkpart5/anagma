
use std::path::Path;

use serde::Deserialize;
use serde::de::Deserializer;

use crate::config::selection::matcher::Matcher;
use crate::config::selection::matcher::Error as MatcherError;

#[derive(Debug)]
pub enum Error {
    CannotBuildMatcher(MatcherError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::CannotBuildMatcher(ref err) => write!(f, "cannot build matcher: {}", err),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Error::CannotBuildMatcher(ref err) => Some(err),
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum MatcherOrIncExc {
    Matcher(Matcher),
    IncExc(IncludeExclude),
}

impl From<MatcherOrIncExc> for Selector {
    fn from(x: MatcherOrIncExc) -> Self {
        match x {
            MatcherOrIncExc::Matcher(m) => Self::from(m),
            MatcherOrIncExc::IncExc(inex) => Self::new(inex.include, inex.exclude),
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct IncludeExclude {
    include: Matcher,
    exclude: Matcher,
}

impl Default for IncludeExclude {
    fn default() -> Self {
        Self {
            include: Matcher::any(),
            exclude: Matcher::empty(),
        }
    }
}

impl From<IncludeExclude> for Selector {
    fn from(ie: IncludeExclude) -> Self {
        Self::new(ie.include, ie.exclude)
    }
}

#[derive(Debug)]
pub struct Selector {
    include: Matcher,
    exclude: Matcher,
}

impl<'de> Deserialize<'de> for Selector {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        use serde::de::Error;
        Ok(MatcherOrIncExc::deserialize(deserializer).map_err(Error::custom)?.into())
    }
}

impl From<Matcher> for Selector {
    fn from(m: Matcher) -> Self {
        Self { include: m, ..Default::default() }
    }
}

impl Default for Selector {
    fn default() -> Self {
        IncludeExclude::default().into()
    }
}

impl Selector {
    pub fn new(include: Matcher, exclude: Matcher) -> Self {
        Selector {
            include,
            exclude,
        }
    }

    /// Convenience method to create a `Selector` from iterables of patterns.
    pub fn from_patterns<II, IE, S>(include_patterns: II, exclude_patterns: IE) -> Result<Self, Error>
    where
        II: IntoIterator<Item = S>,
        IE: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let include = Matcher::build(include_patterns).map_err(Error::CannotBuildMatcher)?;
        let exclude = Matcher::build(exclude_patterns).map_err(Error::CannotBuildMatcher)?;

        Ok(Self::new(include, exclude))
    }

    /// Returns true if the path is a pattern match.
    /// In order to be a pattern match, the path must match the include filter, and must NOT match the exclude filter.
    /// This uses only the lexical content of the path, and does not access the filesystem.
    pub fn is_pattern_match<P: AsRef<Path>>(&self, path: P) -> bool {
        self.include.is_match(&path) && !self.exclude.is_match(&path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let selection = Selector::default();

        assert!(selection.is_pattern_match("all"));
        assert!(selection.is_pattern_match("paths"));
        assert!(selection.is_pattern_match("should"));
        assert!(selection.is_pattern_match("pass"));
        assert!(selection.is_pattern_match("this"));
        assert!(selection.is_pattern_match("selection"));
        assert!(selection.is_pattern_match("a/b/c"));
    }

    #[test]
    fn test_deserialization() {
        // Bare single pattern.
        let text = "'*.ogg'";
        let sel = serde_yaml::from_str::<Selector>(&text).unwrap();
        assert_eq!(sel.is_pattern_match("path/to/music.ogg"), true);
        assert_eq!(sel.is_pattern_match("path/to/music.mp3"), false);
        assert_eq!(sel.is_pattern_match("path/to/audio.ogg"), true);
        assert_eq!(sel.is_pattern_match("path/to/audio.mp3"), false);
        assert_eq!(sel.is_pattern_match("path/to/other.abc"), false);

        // Bare multiple patterns.
        let text = "['*.ogg', 'music*']";
        let sel = serde_yaml::from_str::<Selector>(&text).unwrap();
        assert_eq!(sel.is_pattern_match("path/to/music.ogg"), true);
        assert_eq!(sel.is_pattern_match("path/to/music.mp3"), true);
        assert_eq!(sel.is_pattern_match("path/to/audio.ogg"), true);
        assert_eq!(sel.is_pattern_match("path/to/audio.mp3"), false);
        assert_eq!(sel.is_pattern_match("path/to/other.abc"), false);

        // Single pattern for each of include and exclude.
        let text = "{ include: '*.ogg', exclude: 'audio*' }";
        let sel = serde_yaml::from_str::<Selector>(&text).unwrap();
        assert_eq!(sel.is_pattern_match("path/to/music.ogg"), true);
        assert_eq!(sel.is_pattern_match("path/to/music.mp3"), false);
        assert_eq!(sel.is_pattern_match("path/to/audio.ogg"), false);
        assert_eq!(sel.is_pattern_match("path/to/audio.mp3"), false);
        assert_eq!(sel.is_pattern_match("path/to/other.abc"), false);

        // Multiple patterns for each of include and exclude.
        let text = "{ include: ['*.ogg', 'music*'], exclude: ['*.mp3', 'audio*'] }";
        let sel = serde_yaml::from_str::<Selector>(&text).unwrap();
        assert_eq!(sel.is_pattern_match("path/to/music.ogg"), true);
        assert_eq!(sel.is_pattern_match("path/to/music.mp3"), false);
        assert_eq!(sel.is_pattern_match("path/to/audio.ogg"), false);
        assert_eq!(sel.is_pattern_match("path/to/audio.mp3"), false);
        assert_eq!(sel.is_pattern_match("path/to/other.abc"), false);

        // Using default value for missing include patterns.
        let text = "exclude: ['*.mp3', 'audio*']";
        let sel = serde_yaml::from_str::<Selector>(&text).unwrap();
        assert_eq!(sel.is_pattern_match("path/to/music.ogg"), true);
        assert_eq!(sel.is_pattern_match("path/to/music.mp3"), false);
        assert_eq!(sel.is_pattern_match("path/to/audio.ogg"), false);
        assert_eq!(sel.is_pattern_match("path/to/audio.mp3"), false);
        assert_eq!(sel.is_pattern_match("path/to/other.abc"), true);

        // Using default value for missing exclude patterns.
        let text = "include: ['*.ogg', 'music*']";
        let sel = serde_yaml::from_str::<Selector>(&text).unwrap();
        assert_eq!(sel.is_pattern_match("path/to/music.ogg"), true);
        assert_eq!(sel.is_pattern_match("path/to/music.mp3"), true);
        assert_eq!(sel.is_pattern_match("path/to/audio.ogg"), true);
        assert_eq!(sel.is_pattern_match("path/to/audio.mp3"), false);
        assert_eq!(sel.is_pattern_match("path/to/other.abc"), false);
    }

    #[test]
    fn test_from_patterns() {
        // Positive test cases.
        assert!(Selector::from_patterns(&["*"], &["*"]).is_ok());
        assert!(Selector::from_patterns(&["*.a", "*.b"], &["*.a", "*.b"]).is_ok());
        assert!(Selector::from_patterns(&["?.a", "?.b"], &["?.a", "?.b"]).is_ok());
        assert!(Selector::from_patterns(&["*.a"], &["*.a"]).is_ok());
        assert!(Selector::from_patterns(&["**"], &["**"]).is_ok());
        assert!(Selector::from_patterns(&["a/**/b"], &["a/**/b"]).is_ok());
        assert!(Selector::from_patterns(&[""; 0], &[""; 0]).is_ok());
        assert!(Selector::from_patterns(&[""], &[""]).is_ok());
        assert!(Selector::from_patterns(&["[a-z]*.a"], &["[a-z]*.a"]).is_ok());
        assert!(Selector::from_patterns(&["**", "[a-z]*.a"], &["**", "[a-z]*.a"]).is_ok());
        assert!(Selector::from_patterns(&["[!abc]"], &["[!abc]"]).is_ok());
        assert!(Selector::from_patterns(&["[*]"], &["[*]"]).is_ok());
        assert!(Selector::from_patterns(&["[?]"], &["[?]"]).is_ok());
        assert!(Selector::from_patterns(&["{*.a,*.b,*.c}"], &["{*.a,*.b,*.c}"]).is_ok());

        // Negative test cases.
        // Invalid double star.
        // assert!(Selector::from_patterns(&["a**b"], &["a**b"]).is_err());
        // Unclosed character class.
        assert!(Selector::from_patterns(&["[abc"], &["[abc"]).is_err());
        // Malformed character range.
        assert!(Selector::from_patterns(&["[z-a]"], &["[z-a]"]).is_err());
        // Unclosed alternates.
        assert!(Selector::from_patterns(&["{*.a,*.b,*.c"], &["{*.a,*.b,*.c"]).is_err());
        // Unopened alternates.
        // assert!(Selector::from_patterns(&["*.a,*.b,*.c}"], &["*.a,*.b,*.c}"]).is_err());
        // Nested alternates.
        assert!(Selector::from_patterns(&["{*.a,{*.b,*.c}}"], &["{*.a,{*.b,*.c}}"]).is_err());
        // Dangling escape.
        assert!(Selector::from_patterns(&["*.a\\"], &["*.a\\"]).is_err());
    }

    #[test]
    fn test_is_pattern_match() {
        let selection = Selector::from_patterns(&["*.flac"], &["*.mp3"]).unwrap();

        assert_eq!(selection.is_pattern_match("path/to/music.flac"), true);
        assert_eq!(selection.is_pattern_match("path/to/music.mp3"), false);
        assert_eq!(selection.is_pattern_match("path/to/music.wav"), false);

        let selection = Selector::from_patterns(&["*.flac", "*.wav"], &["*.mp3", "*.ogg"]).unwrap();

        assert_eq!(selection.is_pattern_match("path/to/music.flac"), true);
        assert_eq!(selection.is_pattern_match("path/to/music.wav"), true);
        assert_eq!(selection.is_pattern_match("path/to/music.mp3"), false);
        assert_eq!(selection.is_pattern_match("path/to/music.ogg"), false);
        assert_eq!(selection.is_pattern_match("path/to/music.aac"), false);

        let selection = Selector::from_patterns(&["*"], &["*.mp3", "*.ogg"]).unwrap();

        assert_eq!(selection.is_pattern_match("path/to/music.flac"), true);
        assert_eq!(selection.is_pattern_match("path/to/music.wav"), true);
        assert_eq!(selection.is_pattern_match("path/to/music.aac"), true);
        assert_eq!(selection.is_pattern_match("path/to/music.mpc"), true);
        assert_eq!(selection.is_pattern_match("path/to/music.mp3"), false);
        assert_eq!(selection.is_pattern_match("path/to/music.ogg"), false);

        let selection = Selector::from_patterns(&["*.flac", "*.wav"], &["item*", "self*"]).unwrap();

        assert_eq!(selection.is_pattern_match("path/to/music.flac"), true);
        assert_eq!(selection.is_pattern_match("path/to/music.wav"), true);
        assert_eq!(selection.is_pattern_match("path/to/item.flac"), false);
        assert_eq!(selection.is_pattern_match("path/to/self.flac"), false);
        assert_eq!(selection.is_pattern_match("path/to/music.aac"), false);
        assert_eq!(selection.is_pattern_match("path/to/music.mpc"), false);
        assert_eq!(selection.is_pattern_match("path/to/music.mp3"), false);
        assert_eq!(selection.is_pattern_match("path/to/music.ogg"), false);
    }
}
