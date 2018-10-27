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
use library::selection::matcher::Matcher;

#[derive(Deserialize)]
#[serde(default)]
pub struct Config {
    pub include: Matcher,
    pub exclude: Matcher,
    pub sort_order: SortOrder,
    pub item_fn: String,
    pub self_fn: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            include: Matcher::any(),
            exclude: Matcher::from_patterns(&["*.yml"]).unwrap(),
            sort_order: SortOrder::Name,
            item_fn: String::from("item.yml"),
            self_fn: String::from("self.yml"),
        }
    }
}

impl Config {
    /// Returns true if the path matches the pattern of this config.
    /// This only uses the lexical content of the path.
    pub fn is_pattern_match<P: AsRef<Path>>(&self, path: P) -> bool {
        self.include.is_match(&path) && !self.exclude.is_match(&path)
    }

    /// Returns true if a path is selected as part of this config.
    /// Directories are always marked as included.
    /// Files are included if they meet the pattern criteria.
    pub fn is_selected<P: AsRef<Path>>(&self, path: P) -> bool {
        path.as_ref().is_dir() || (path.as_ref().is_file() && self.is_pattern_match(path))
    }

    // NOTE: Sorting is now only done during plexing.
    /// Returns items from the input that are selected according to this config.
    pub fn select<'a, II, P>(&'a self, item_paths: II) -> impl Iterator<Item = P> + 'a
    where
        II: IntoIterator<Item = P>,
        II::IntoIter: 'a,
        P: AsRef<Path>,
    {
        let filtered = item_paths
            .into_iter()
            .filter(move |ip| self.is_selected(ip));

        filtered
    }

    pub fn select_in_dir<'a, P>(&'a self, dir_path: P) -> Result<impl Iterator<Item = PathBuf> + 'a, Error>
    where
        P: AsRef<Path>,
    {
        let item_entries = dir_path
            .as_ref()
            .read_dir().context(ErrorKind::CannotReadDir)?
            .collect::<Result<Vec<_>, _>>().context(ErrorKind::CannotReadDirEntry)?;

        let sel_item_paths = self.select::<'a>(item_entries.into_iter().map(|entry| entry.path()));

        Ok(sel_item_paths)
    }
}

#[cfg(test)]
mod tests {
    use serde_yaml;

    use library::selection::matcher::Matcher;

    use super::Config;
    use super::SortOrder;

    #[test]
    fn test_is_pattern_match() {
        let config = Config {
            include: Matcher::from_patterns(&["*.flac", "*.mp3"]).unwrap(),
            exclude: Matcher::from_patterns(&["*.yml", "*.jpg"]).unwrap(),
            ..Default::default()
        };

        assert_eq!(config.is_pattern_match("music.flac"), true);
        assert_eq!(config.is_pattern_match("music.mp3"), true);
        assert_eq!(config.is_pattern_match("photo.png"), false);
        assert_eq!(config.is_pattern_match("self.yml"), false);
        assert_eq!(config.is_pattern_match("unknown"), false);
    }

    #[test]
    fn test_deserialization() {
        let text_config = "include: '*.flac'\nsort_order: name";

        let config: Config = serde_yaml::from_str(&text_config).unwrap();

        assert!(config.include.is_match("music.flac"));
        assert!(!config.include.is_match("music.mp3"));
        assert!(!config.include.is_match("photo.png"));
        assert!(config.exclude.is_match("self.yml"));
        assert!(config.exclude.is_match("item.yml"));
        assert!(!config.exclude.is_match("music.flac"));
        assert_eq!(config.sort_order, SortOrder::Name);
        assert_eq!(config.item_fn, "item.yml");
        assert_eq!(config.self_fn, "self.yml");

        let text_config = "include:\n  - '*.flac'\n  - '*.mp3'\nsort_order: mod_time";

        let config: Config = serde_yaml::from_str(&text_config).unwrap();

        assert!(config.include.is_match("music.flac"));
        assert!(config.include.is_match("music.mp3"));
        assert!(!config.include.is_match("photo.png"));
        assert_eq!(config.sort_order, SortOrder::ModTime);
        assert_eq!(config.item_fn, "item.yml");
        assert_eq!(config.self_fn, "self.yml");

        let text_config = "include: '*'\nsort_order: mod_time";

        let config: Config = serde_yaml::from_str(&text_config).unwrap();

        assert!(config.include.is_match("music.flac"));
        assert!(config.include.is_match("music.mp3"));
        assert!(config.include.is_match("photo.png"));
        assert_eq!(config.sort_order, SortOrder::ModTime);
        assert_eq!(config.item_fn, "item.yml");
        assert_eq!(config.self_fn, "self.yml");

        let text_config = "include: '*'
sort_order: name
item_fn: item_meta.yml
";

        let config: Config = serde_yaml::from_str(&text_config).unwrap();

        assert!(config.include.is_match("music.flac"));
        assert!(config.include.is_match("music.mp3"));
        assert!(config.include.is_match("photo.png"));
        assert_eq!(config.sort_order, SortOrder::Name);
        assert_eq!(config.item_fn, "item_meta.yml");
        assert_eq!(config.self_fn, "self.yml");
    }
}
