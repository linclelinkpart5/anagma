//! Provides configuration options for a library, both programmatically and via YAML files.

use std::collections::HashMap;

use library::sort_order::SortOrder;
use library::selection::Selection;
use metadata::reader::MetaFormat;
use metadata::aggregator::AggMethod;

#[derive(Deserialize)]
#[serde(default)]
pub struct Config {
    pub selection: Selection,
    pub sort_order: SortOrder,
    pub item_fn: String,
    pub self_fn: String,
    pub meta_format: MetaFormat,
    pub agg_methods: HashMap<String, AggMethod>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            selection: Selection::default(),
            sort_order: SortOrder::default(),
            item_fn: String::from("item.yml"),
            self_fn: String::from("self.yml"),
            meta_format: MetaFormat::default(),
            agg_methods: HashMap::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_yaml;

    use library::selection::Selection;
    use metadata::reader::MetaFormat;

    use super::Config;
    use super::SortOrder;

    #[test]
    fn test_deserialization() {
        let text_config = "selection:\n  include: '*.flac'\nsort_order: name";

        let config: Config = serde_yaml::from_str(&text_config).unwrap();

        assert!(config.selection.is_pattern_match("music.flac"));
        assert!(!config.selection.is_pattern_match("music.mp3"));
        assert!(!config.selection.is_pattern_match("photo.png"));
        assert!(!config.selection.is_pattern_match("self.yml"));
        assert!(!config.selection.is_pattern_match("item.yml"));
        assert_eq!(config.sort_order, SortOrder::Name);
        assert_eq!(config.item_fn, "item.yml");
        assert_eq!(config.self_fn, "self.yml");
        assert_eq!(config.meta_format, MetaFormat::Yaml);

        let text_config = "selection:\n  include:\n    - '*.flac'\n    - '*.mp3'\nsort_order: mod_time";

        let config: Config = serde_yaml::from_str(&text_config).unwrap();

        assert!(config.selection.is_pattern_match("music.flac"));
        assert!(config.selection.is_pattern_match("music.mp3"));
        assert!(!config.selection.is_pattern_match("photo.png"));
        assert_eq!(config.sort_order, SortOrder::ModTime);
        assert_eq!(config.item_fn, "item.yml");
        assert_eq!(config.self_fn, "self.yml");
        assert_eq!(config.meta_format, MetaFormat::Yaml);

        let text_config = "selection:\n  include: '*'\nsort_order: mod_time";

        let config: Config = serde_yaml::from_str(&text_config).unwrap();

        assert!(config.selection.is_pattern_match("music.flac"));
        assert!(config.selection.is_pattern_match("music.mp3"));
        assert!(config.selection.is_pattern_match("photo.png"));
        assert_eq!(config.sort_order, SortOrder::ModTime);
        assert_eq!(config.item_fn, "item.yml");
        assert_eq!(config.self_fn, "self.yml");
        assert_eq!(config.meta_format, MetaFormat::Yaml);

        let text_config = "selection:
  include: '*'
  exclude: '*.mp3'
sort_order: name
item_fn: item_meta.yml
meta_format: yaml
";

        let config: Config = serde_yaml::from_str(&text_config).unwrap();

        assert!(config.selection.is_pattern_match("music.flac"));
        assert!(!config.selection.is_pattern_match("music.mp3"));
        assert!(config.selection.is_pattern_match("photo.png"));
        assert_eq!(config.sort_order, SortOrder::Name);
        assert_eq!(config.item_fn, "item_meta.yml");
        assert_eq!(config.self_fn, "self.yml");
        assert_eq!(config.meta_format, MetaFormat::Yaml);
    }
}
