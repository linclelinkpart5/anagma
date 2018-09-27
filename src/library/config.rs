//! Provides configuration options for a Taggu library, both programmatically and via YAML files.
use std::path::Path;

use failure::Error;
use serde::Deserialize;
use serde::de::Deserializer;

use library::sort_order::SortOrder;
use library::selection::Selection;

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

fn coerce_to_selection<'de, D>(deserializer: D) -> Result<Selection, D::Error>
where D: Deserializer<'de> {
    use serde::de::Error;
    let oom_patterns = OneOrManyPatterns::deserialize(deserializer).map_err(Error::custom)?;
    let selection = oom_patterns.into_selection().map_err(Error::custom)?;
    Ok(selection)
}

#[derive(Deserialize)]
#[serde(default)]
pub struct Config {
    #[serde(deserialize_with = "coerce_to_selection")]
    pub include: Selection,
    #[serde(deserialize_with = "coerce_to_selection")]
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
    pub fn is_selected<P: AsRef<Path>>(&self, path: P) -> bool {
        self.include.is_match(&path) && !self.exclude.is_match(&path)
    }
}

#[cfg(test)]
mod tests {
    use serde_yaml;

    use super::Config;
    use super::SortOrder;

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
