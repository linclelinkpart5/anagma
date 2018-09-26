//! Provides configuration options for a Taggu library, both programmatically and via YAML files.

use globset::Glob;
use globset::GlobSet;
use globset::GlobSetBuilder;
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

fn default_item_meta_fn() -> String {
    "item.yml".to_string()
}

fn default_self_meta_fn() -> String {
    "self.yml".to_string()
}

#[derive(Deserialize)]
pub struct Config {
    #[serde(deserialize_with = "coerce_to_selection")]
    pub selection: Selection,
    pub sort_order: SortOrder,
    #[serde(default = "default_item_meta_fn")]
    pub item_meta_fn: String,
    #[serde(default = "default_self_meta_fn")]
    pub self_meta_fn: String,
}

impl Default for Config {
    fn default() -> Self {
        // TODO: Change to be star after testing.
        let selection = Selection::from_patterns(&["*.flac"]).unwrap();
        // let selection = Selection::any();

        Config {
            selection,
            sort_order: SortOrder::Name,
            item_meta_fn: default_item_meta_fn(),
            self_meta_fn: default_self_meta_fn(),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_yaml;

    use super::Config;
    use super::SortOrder;

    #[test]
    fn test_deserialization() {
        let text_config = "selection: '*.flac'\nsort_order: name";

        let config: Config = serde_yaml::from_str(&text_config).unwrap();

        assert!(config.selection.is_match("music.flac"));
        assert!(!config.selection.is_match("music.mp3"));
        assert!(!config.selection.is_match("photo.png"));
        assert_eq!(config.sort_order, SortOrder::Name);
        assert_eq!(config.item_meta_fn, "item.yml");
        assert_eq!(config.self_meta_fn, "self.yml");

        let text_config = "selection:\n  - '*.flac'\n  - '*.mp3'\nsort_order: mod_time";

        let config: Config = serde_yaml::from_str(&text_config).unwrap();

        assert!(config.selection.is_match("music.flac"));
        assert!(config.selection.is_match("music.mp3"));
        assert!(!config.selection.is_match("photo.png"));
        assert_eq!(config.sort_order, SortOrder::ModTime);
        assert_eq!(config.item_meta_fn, "item.yml");
        assert_eq!(config.self_meta_fn, "self.yml");

        let text_config = "selection: '*'\nsort_order: mod_time";

        let config: Config = serde_yaml::from_str(&text_config).unwrap();

        assert!(config.selection.is_match("music.flac"));
        assert!(config.selection.is_match("music.mp3"));
        assert!(config.selection.is_match("photo.png"));
        assert_eq!(config.sort_order, SortOrder::ModTime);
        assert_eq!(config.item_meta_fn, "item.yml");
        assert_eq!(config.self_meta_fn, "self.yml");

        let text_config = "selection: '*'
sort_order: name
item_meta_fn: item_meta.yml
";

        let config: Config = serde_yaml::from_str(&text_config).unwrap();

        assert!(config.selection.is_match("music.flac"));
        assert!(config.selection.is_match("music.mp3"));
        assert!(config.selection.is_match("photo.png"));
        assert_eq!(config.sort_order, SortOrder::Name);
        assert_eq!(config.item_meta_fn, "item_meta.yml");
        assert_eq!(config.self_meta_fn, "self.yml");
    }
}
