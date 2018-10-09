//! Provides configuration options for a Taggu library, both programmatically and via YAML files.

use std::path::Path;
use std::path::PathBuf;

use failure::Error;
use failure::ResultExt;

use error::ErrorKind;
use library::sort_order::SortOrder;
use library::selection::Selection;

#[derive(Deserialize)]
#[serde(default)]
pub struct Config {
    pub include: Selection,
    pub exclude: Selection,
    pub sort_order: SortOrder,
    pub item_fn: String,
    pub self_fn: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            include: Selection::any(),
            exclude: Selection::from_patterns(&["*.yml"]).unwrap(),
            sort_order: SortOrder::Name,
            item_fn: String::from("item.yml"),
            self_fn: String::from("self.yml"),
        }
    }
}

impl Config {
    /// Indicates if a path is selected as part of this config.
    /// This only uses the lexical content of the path.
    pub fn is_pattern_match<P: AsRef<Path>>(&self, path: P) -> bool {
        self.include.is_match(&path) && !self.exclude.is_match(&path)
    }

    /// Indicates if a path is selected as part of this config.
    /// Directories are always marked as included.
    pub fn is_selected<P: AsRef<Path>>(&self, path: P) -> bool {
        path.as_ref().is_dir() || (path.as_ref().is_file() && self.is_pattern_match(path))
    }

    // NOTE: Sorting is now only done during plexing.
    pub fn select<II, P>(&self, item_paths: II) -> Vec<P>
    where
        II: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        item_paths
            .into_iter()
            .filter(|ip| self.is_selected(ip))
            .collect()
    }

    pub fn select_in_dir<P>(&self, dir_path: P) -> Result<Vec<PathBuf>, Error>
    where
        P: AsRef<Path>,
    {
        let item_entries = dir_path
            .as_ref()
            .read_dir().map_err(|_| ErrorKind::CannotReadDir(dir_path.as_ref().to_path_buf()))?
            .collect::<Result<Vec<_>, _>>().map_err(|_| ErrorKind::CannotReadDirEntry)?;

        let item_paths = self.select(item_entries.into_iter().map(|entry| entry.path()));

        Ok(item_paths)
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
            include: Selection::from_patterns(&["*.flac", "*.mp3"]).unwrap(),
            exclude: Selection::from_patterns(&["*.yml", "*.jpg"]).unwrap(),
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
