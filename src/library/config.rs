//! Provides configuration options for a Taggu library, both programmatically and via YAML files.

use globset::Glob;
use globset::GlobSet;
use globset::GlobSetBuilder;
use failure::Error;
use serde::Deserialize;
use serde::de::Deserializer;

use library::sort_order::SortOrder;

#[derive(Deserialize)]
#[serde(untagged)]
enum GlobStrings {
    One(String),
    Many(Vec<String>),
}

impl GlobStrings {
    fn make_globset(&self) -> Result<GlobSet, Error> {
        let mut builder = GlobSetBuilder::new();

        match *self {
            GlobStrings::One(ref pattern) => {
                builder.add(Glob::new(&pattern)?);
            },
            GlobStrings::Many(ref patterns) => {
                for pattern in patterns {
                    builder.add(Glob::new(&pattern)?);
                }
            },
        }

        Ok(builder.build()?)
    }
}

fn coerce_to_globset<'de, D>(deserializer: D) -> Result<GlobSet, D::Error>
where D: Deserializer<'de> {
    use serde::de::Error;
    let glob_strings = GlobStrings::deserialize(deserializer).map_err(Error::custom)?;
    let glob_set = glob_strings.make_globset().map_err(Error::custom)?;
    Ok(glob_set)
}

fn default_item_meta_fn() -> String {
    "item.yml".to_string()
}

fn default_self_meta_fn() -> String {
    "self.yml".to_string()
}

#[derive(Deserialize)]
pub struct Config {
    #[serde(deserialize_with = "coerce_to_globset")]
    pub selection: GlobSet,
    pub sort_order: SortOrder,
    #[serde(default = "default_item_meta_fn")]
    pub item_meta_fn: String,
    #[serde(default = "default_self_meta_fn")]
    pub self_meta_fn: String,
}

impl Default for Config {
    fn default() -> Self {
        // TODO: Change to be star after testing.
        let mut builder = GlobSetBuilder::new();
        builder.add(Glob::new("*.flac").unwrap());

        let selection = builder.build().unwrap();

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
