//! Provides configuration options for a library, both programmatically and via config files.

pub mod selection;
pub mod sorter;

use std::convert::{TryFrom, TryInto};
use std::path::Path;

use serde::Deserialize;

use self::selection::Selection;
use self::sorter::Sorter;

use crate::metadata::schema::SchemaFormat;
use crate::source::{Anchor, Source, CreateError as SourceCreateError};

const DEFAULT_INTERNAL_STUB: &str = "album";
const DEFAULT_EXTERNAL_STUB: &str = "track";

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Sources {
    external: Vec<String>,
    internal: Vec<String>,
}

impl Default for Sources {
    fn default() -> Self {
        let default_fmt = SchemaFormat::Json;
        let default_ext = default_fmt.as_ref();

        let external = vec![format!("{}.{}", DEFAULT_EXTERNAL_STUB, default_ext)];
        let internal = vec![format!("{}.{}", DEFAULT_INTERNAL_STUB, default_ext)];

        Self { external, internal, }
    }
}

#[derive(Default, Deserialize)]
#[serde(default)]
pub struct ConfigRepr {
    pub filtering: Selection,
    pub ordering: Sorter,
    pub sources: Sources,
}

impl TryFrom<ConfigRepr> for Config {
    type Error = SourceCreateError;

    fn try_from(value: ConfigRepr) -> Result<Self, Self::Error> {
        let mut sources = Vec::new();

        for name in value.sources.external {
            let src = Source::from_name(name, Anchor::External)?;
            sources.push(src);
        }

        for name in value.sources.internal {
            let src = Source::from_name(name, Anchor::Internal)?;
            sources.push(src);
        }

        Ok(Self {
            selection: value.filtering,
            sorter: value.ordering,
            sources,
        })
    }
}
#[derive(Deserialize)]
#[serde(try_from = "ConfigRepr")]
pub struct Config {
    pub selection: Selection,
    pub sorter: Sorter,
    pub sources: Vec<Source>,
}

impl Default for Config {
    fn default() -> Self {
        // NOTE: This is expected to never fail.
        TryInto::<_>::try_into(ConfigRepr::default()).unwrap()
    }
}

impl Config {
    pub fn from_str(value: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config = toml::from_str(&value)?;
        Ok(config)
    }

    pub fn from_file<P: AsRef<Path>>(path: &P) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(path)?;
        Self::from_str(&contents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::config::sorter::sort_by::SortBy;

    use str_macro::str;

    #[test]
    fn deserialization() {
        let text_config = r#"
            [filtering]
            include_files = "*.flac"
            [ordering]
            sort_by = "name"
        "#;

        let config: Config = toml::from_str(&text_config).unwrap();

        assert_eq!(config.selection.is_file_pattern_match(&"music.flac"), true);
        assert_eq!(config.selection.is_file_pattern_match(&"music.mp3"), false);
        assert_eq!(config.selection.is_file_pattern_match(&"photo.png"), false);
        assert_eq!(config.selection.is_file_pattern_match(&"self.yml"), false);
        assert_eq!(config.selection.is_file_pattern_match(&"item.yml"), false);
        assert_eq!(config.sorter.sort_by, SortBy::Name);
        assert_eq!(
            config.sources,
            vec![
                Source::from_name(str!("track.json"), Anchor::External).unwrap(),
                Source::from_name(str!("album.json"), Anchor::Internal).unwrap(),
            ]
        );

        let text_config = r#"
            [filtering]
            include_files = ["*.flac", "*.mp3"]
            [ordering]
            sort_by = "mod_time"
        "#;

        let config: Config = toml::from_str(&text_config).unwrap();

        assert_eq!(config.selection.is_file_pattern_match(&"music.flac"), true);
        assert_eq!(config.selection.is_file_pattern_match(&"music.mp3"), true);
        assert_eq!(config.selection.is_file_pattern_match(&"photo.png"), false);
        assert_eq!(config.sorter.sort_by, SortBy::ModTime);
        assert_eq!(
            config.sources,
            vec![
                Source::from_name(str!("track.json"), Anchor::External).unwrap(),
                Source::from_name(str!("album.json"), Anchor::Internal).unwrap(),
            ]
        );

        let text_config = r#"
            [filtering]
            include_files = "*"
            [ordering]
            sort_by = "mod_time"
        "#;

        let config: Config = toml::from_str(&text_config).unwrap();

        assert_eq!(config.selection.is_file_pattern_match(&"music.flac"), true);
        assert_eq!(config.selection.is_file_pattern_match(&"music.mp3"), true);
        assert_eq!(config.selection.is_file_pattern_match(&"photo.png"), true);
        assert_eq!(config.sorter.sort_by, SortBy::ModTime);
        assert_eq!(
            config.sources,
            vec![
                Source::from_name(str!("track.json"), Anchor::External).unwrap(),
                Source::from_name(str!("album.json"), Anchor::Internal).unwrap(),
            ]
        );

        let text_config = r#"
            [filtering]
            include_files = "*"
            exclude_files = "*.mp3"
            [ordering]
            sort_by = "name"
            [sources]
            external = ["item_meta.yml"]
        "#;

        let config: Config = toml::from_str(&text_config).unwrap();

        assert_eq!(config.selection.is_file_pattern_match(&"music.flac"), true);
        assert_eq!(config.selection.is_file_pattern_match(&"music.mp3"), false);
        assert_eq!(config.selection.is_file_pattern_match(&"photo.png"), true);
        assert_eq!(config.sorter.sort_by, SortBy::Name);
        assert_eq!(
            config.sources,
            vec![
                Source::from_name(str!("item_meta.yml"), Anchor::External).unwrap(),
                Source::from_name(str!("album.json"), Anchor::Internal).unwrap(),
            ]
        );
    }
}
