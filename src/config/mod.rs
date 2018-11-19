//! Provides configuration options for a library, both programmatically and via config files.

pub mod agg_method;
pub mod inherit_method;
pub mod meta_format;
pub mod selection;
pub mod sort_order;

use std::collections::HashMap;

use config::agg_method::AggMethod;
use config::meta_format::MetaFormat;
use config::selection::Selection;
use config::sort_order::SortOrder;

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
        use metadata::location::MetaLocation;

        // TODO: Is there a way to intelligently populate this while also preserving defaulting behavior?
        let selection = Selection::default();
        let sort_order = SortOrder::default();
        let meta_format = MetaFormat::default();
        let agg_methods = HashMap::default();
        let item_fn = format!("{}.{}", MetaLocation::Siblings.default_file_name(), meta_format.default_file_extension());
        let self_fn = format!("{}.{}", MetaLocation::Contains.default_file_name(), meta_format.default_file_extension());

        Config {
            selection,
            sort_order,
            item_fn,
            self_fn,
            meta_format,
            agg_methods,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_yaml;

    use config::Config;
    use config::sort_order::SortOrder;
    use config::meta_format::MetaFormat;

    #[test]
    fn test_deserialization() {
        let text_config = "selection:\n  include: '*.flac'\nsort_order: name";

        let config: Config = serde_yaml::from_str(&text_config).unwrap();

        assert_eq!(config.selection.is_pattern_match("music.flac"), true);
        assert_eq!(config.selection.is_pattern_match("music.mp3"), false);
        assert_eq!(config.selection.is_pattern_match("photo.png"), false);
        assert_eq!(config.selection.is_pattern_match("self.yml"), false);
        assert_eq!(config.selection.is_pattern_match("item.yml"), false);
        assert_eq!(config.sort_order, SortOrder::Name);
        assert_eq!(config.item_fn, "item.yml");
        assert_eq!(config.self_fn, "self.yml");
        assert_eq!(config.meta_format, MetaFormat::Yaml);

        let text_config = "selection:\n  include:\n    - '*.flac'\n    - '*.mp3'\nsort_order: mod_time";

        let config: Config = serde_yaml::from_str(&text_config).unwrap();

        assert_eq!(config.selection.is_pattern_match("music.flac"), true);
        assert_eq!(config.selection.is_pattern_match("music.mp3"), true);
        assert_eq!(config.selection.is_pattern_match("photo.png"), false);
        assert_eq!(config.sort_order, SortOrder::ModTime);
        assert_eq!(config.item_fn, "item.yml");
        assert_eq!(config.self_fn, "self.yml");
        assert_eq!(config.meta_format, MetaFormat::Yaml);

        let text_config = "selection:\n  include: '*'\nsort_order: mod_time";

        let config: Config = serde_yaml::from_str(&text_config).unwrap();

        assert_eq!(config.selection.is_pattern_match("music.flac"), true);
        assert_eq!(config.selection.is_pattern_match("music.mp3"), true);
        assert_eq!(config.selection.is_pattern_match("photo.png"), true);
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

        assert_eq!(config.selection.is_pattern_match("music.flac"), true);
        assert_eq!(config.selection.is_pattern_match("music.mp3"), false);
        assert_eq!(config.selection.is_pattern_match("photo.png"), true);
        assert_eq!(config.sort_order, SortOrder::Name);
        assert_eq!(config.item_fn, "item_meta.yml");
        assert_eq!(config.self_fn, "self.yml");
        assert_eq!(config.meta_format, MetaFormat::Yaml);
    }
}
