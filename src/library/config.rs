//! Provides configuration options for a Taggu library, both programmatically and via YAML files.

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
    #[fail(display = "cannot read directory")]
    CannotReadDir,
    #[fail(display = "cannot read directory entry")]
    CannotReadDirEntry,
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

use library::sort_order::SortOrder;
use library::selection::Selection;
use library::selection::Matcher;

#[derive(Deserialize)]
#[serde(default)]
pub struct Config {
    pub selection: Selection,
    pub sort_order: SortOrder,
    pub item_fn: String,
    pub self_fn: String,
}

impl Default for Config {
    fn default() -> Self {
        let selection = Selection::from_matchers(Matcher::any(), Matcher::from_patterns(&["*.yml"]).unwrap());
        Config {
            selection: selection,
            sort_order: SortOrder::Name,
            item_fn: String::from("item.yml"),
            self_fn: String::from("self.yml"),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_yaml;

    use library::selection::Selection;

    use super::Config;
    use super::SortOrder;

    #[test]
    fn test_is_pattern_match() {
        let config = Config {
            selection: Selection::from_patterns(
                &["*.flac", "*.mp3"],
                &["*.yml", "*.jpg"],
            ).unwrap(),
            ..Default::default()
        };

        assert_eq!(config.selection.is_pattern_match("music.flac"), true);
        assert_eq!(config.selection.is_pattern_match("music.mp3"), true);
        assert_eq!(config.selection.is_pattern_match("photo.png"), false);
        assert_eq!(config.selection.is_pattern_match("self.yml"), false);
        assert_eq!(config.selection.is_pattern_match("unknown"), false);
    }

//     #[test]
//     fn test_deserialization() {
//         let text_config = "selection:\n  include: '*.flac'\nsort_order: name";

//         let config: Config = serde_yaml::from_str(&text_config).unwrap();

//         assert!(config.selection.is_pattern_match("music.flac"));
//         assert!(!config.selection.is_pattern_match("music.mp3"));
//         assert!(!config.selection.is_pattern_match("photo.png"));
//         assert!(!config.selection.is_pattern_match("self.yml"));
//         assert!(!config.selection.is_pattern_match("item.yml"));
//         assert_eq!(config.sort_order, SortOrder::Name);
//         assert_eq!(config.item_fn, "item.yml");
//         assert_eq!(config.self_fn, "self.yml");

//         let text_config = "selection:\n  include:\n    - '*.flac'\n    - '*.mp3'\nsort_order: mod_time";

//         let config: Config = serde_yaml::from_str(&text_config).unwrap();

//         assert!(config.selection.is_pattern_match("music.flac"));
//         assert!(config.selection.is_pattern_match("music.mp3"));
//         assert!(!config.selection.is_pattern_match("photo.png"));
//         assert_eq!(config.sort_order, SortOrder::ModTime);
//         assert_eq!(config.item_fn, "item.yml");
//         assert_eq!(config.self_fn, "self.yml");

//         let text_config = "selection:\n  include: '*'\nsort_order: mod_time";

//         let config: Config = serde_yaml::from_str(&text_config).unwrap();

//         assert!(config.selection.is_pattern_match("music.flac"));
//         assert!(config.selection.is_pattern_match("music.mp3"));
//         assert!(config.selection.is_pattern_match("photo.png"));
//         assert_eq!(config.sort_order, SortOrder::ModTime);
//         assert_eq!(config.item_fn, "item.yml");
//         assert_eq!(config.self_fn, "self.yml");

//         let text_config = "selection:
//   include: '*'
// sort_order: name
// item_fn: item_meta.yml
// ";

//         let config: Config = serde_yaml::from_str(&text_config).unwrap();

//         assert!(config.selection.is_pattern_match("music.flac"));
//         assert!(config.selection.is_pattern_match("music.mp3"));
//         assert!(config.selection.is_pattern_match("photo.png"));
//         assert_eq!(config.sort_order, SortOrder::Name);
//         assert_eq!(config.item_fn, "item_meta.yml");
//         assert_eq!(config.self_fn, "self.yml");
//     }
}
