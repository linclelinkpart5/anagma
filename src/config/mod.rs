//! Provides configuration options for a library, both programmatically and via config files.

pub mod fallback_method;
pub mod meta_format;
pub mod selection;
pub mod sort_order;

use serde::Deserialize;
use serde::de::Deserializer;

use config::meta_format::MetaFormat;
use config::selection::Selection;
use config::sort_order::SortOrder;
use config::fallback_method::FallbackSpec;
use config::fallback_method::FallbackSpecRepr;
use config::fallback_method::Fallback;
use config::fallback_method::into_fallback_spec;

#[derive(Deserialize)]
#[serde(default)]
struct ConfigRepr {
    pub selection: Selection,
    pub sort_order: SortOrder,
    pub item_fn: String,
    pub self_fn: String,
    pub meta_format: MetaFormat,
    pub fallbacks: FallbackSpecRepr,
    pub default_fallback: Fallback,
    pub map_root_key: String,
}

impl Default for ConfigRepr {
    fn default() -> Self {
        use metadata::location::MetaLocation;

        // TODO: Is there a way to intelligently populate this while also preserving defaulting behavior?
        let selection = Selection::default();
        let sort_order = SortOrder::default();
        let meta_format = MetaFormat::default();
        let item_fn = format!("{}.{}", MetaLocation::Siblings.default_file_name(), meta_format.default_file_extension());
        let self_fn = format!("{}.{}", MetaLocation::Contains.default_file_name(), meta_format.default_file_extension());
        let fallbacks = FallbackSpecRepr::default();
        let default_fallback = Fallback::default();
        let map_root_key = String::from("~");

        ConfigRepr {
            selection,
            sort_order,
            item_fn,
            self_fn,
            meta_format,
            fallbacks,
            default_fallback,
            map_root_key,
        }
    }
}

pub struct Config {
    pub selection: Selection,
    pub sort_order: SortOrder,
    pub item_fn: String,
    pub self_fn: String,
    pub meta_format: MetaFormat,
    pub fallbacks: FallbackSpec,
    pub default_fallback: Fallback,
    pub map_root_key: String,
}

impl Default for Config {
    fn default() -> Self {
        ConfigRepr::default().into()
    }
}

impl From<ConfigRepr> for Config {
    fn from(other: ConfigRepr) -> Self {
        Config {
            selection: other.selection,
            sort_order: other.sort_order,
            item_fn: other.item_fn,
            self_fn: other.self_fn,
            meta_format: other.meta_format,
            fallbacks: into_fallback_spec(other.fallbacks, &other.map_root_key),
            default_fallback: other.default_fallback,
            map_root_key: other.map_root_key,
        }
    }
}

impl<'de> Deserialize<'de> for Config {
    fn deserialize<D>(deserializer: D) -> Result<Config, D::Error>
    where D: Deserializer<'de> {
        use serde::de::Error;
        let config_repr = ConfigRepr::deserialize(deserializer).map_err(Error::custom)?;
        Ok(config_repr.into())
    }
}

#[cfg(test)]
mod tests {
    use serde_yaml;

    use config::Config;
    use config::sort_order::SortOrder;
    use config::meta_format::MetaFormat;
    use config::fallback_method::FallbackSpecNode;
    use config::fallback_method::Fallback;
    use config::fallback_method::InheritFallback;
    use config::fallback_method::HarvestFallback;

    use metadata::types::MetaKey;

    #[test]
    fn test_deserialization() {
        let text_config = r#"
            selection:
                include: '*.flac'
            sort_order: name
        "#;

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

        let text_config = r#"
            selection:
                include:
                    - '*.flac'
                    - '*.mp3'
            sort_order: mod_time
        "#;

        let config: Config = serde_yaml::from_str(&text_config).unwrap();

        assert_eq!(config.selection.is_pattern_match("music.flac"), true);
        assert_eq!(config.selection.is_pattern_match("music.mp3"), true);
        assert_eq!(config.selection.is_pattern_match("photo.png"), false);
        assert_eq!(config.sort_order, SortOrder::ModTime);
        assert_eq!(config.item_fn, "item.yml");
        assert_eq!(config.self_fn, "self.yml");
        assert_eq!(config.meta_format, MetaFormat::Yaml);

        let text_config = r#"
            selection:
                include: '*'
            sort_order: mod_time
        "#;

        let config: Config = serde_yaml::from_str(&text_config).unwrap();

        assert_eq!(config.selection.is_pattern_match("music.flac"), true);
        assert_eq!(config.selection.is_pattern_match("music.mp3"), true);
        assert_eq!(config.selection.is_pattern_match("photo.png"), true);
        assert_eq!(config.sort_order, SortOrder::ModTime);
        assert_eq!(config.item_fn, "item.yml");
        assert_eq!(config.self_fn, "self.yml");
        assert_eq!(config.meta_format, MetaFormat::Yaml);

        let text_config = r#"
            selection:
                include: '*'
                exclude: '*.mp3'
            sort_order: name
            item_fn: item_meta.yml
            meta_format: yaml
            fallbacks:
                title: override
            default_fallback: collect
            map_root_key: 'null'
        "#;

        let config: Config = serde_yaml::from_str(&text_config).unwrap();

        assert_eq!(config.selection.is_pattern_match("music.flac"), true);
        assert_eq!(config.selection.is_pattern_match("music.mp3"), false);
        assert_eq!(config.selection.is_pattern_match("photo.png"), true);
        assert_eq!(config.sort_order, SortOrder::Name);
        assert_eq!(config.item_fn, "item_meta.yml");
        assert_eq!(config.self_fn, "self.yml");
        assert_eq!(config.meta_format, MetaFormat::Yaml);
        assert_eq!(config.fallbacks, hashmap![
            MetaKey::Str("title".into()) => FallbackSpecNode::Leaf(Fallback::Inherit(InheritFallback::Override)),
        ]);
        assert_eq!(config.default_fallback, Fallback::Harvest(HarvestFallback::Collect));
        assert_eq!(config.map_root_key, "null");
    }
}
