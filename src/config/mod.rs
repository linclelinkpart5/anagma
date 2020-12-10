//! Provides configuration options for a library, both programmatically and via config files.

pub mod format;
pub mod selection;
pub mod sorter;

pub use self::format::{Format, Error as FormatError};
pub use self::selection::Selection;
pub use self::sorter::Sorter;

use std::convert::{TryFrom, TryInto};
use std::path::Path;

use serde::Deserialize;
use thiserror::Error;

use self::selection::{SelectionRepr, MatcherError};

use crate::source::{Anchor, Source, Sourcer, CreateError as SourceCreateError};

const DEFAULT_INTERNAL_STUB: &str = "album";
const DEFAULT_EXTERNAL_STUB: &str = "track";

#[derive(Debug, Error)]
pub enum Error {
    #[error("error deserializing matcher: {0}")]
    Matcher(#[from] MatcherError),
    #[error("error deserializing source: {0}")]
    Source(#[from] SourceCreateError),
}

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SourcesRepr {
    #[serde(rename = "track")]
    external: Vec<String>,
    #[serde(rename = "album")]
    internal: Vec<String>,
}

impl Default for SourcesRepr {
    fn default() -> Self {
        let default_fmt = Format::Json;
        let default_ext = default_fmt.as_ref();

        let external = vec![format!("{}.{}", DEFAULT_EXTERNAL_STUB, default_ext)];
        let internal = vec![format!("{}.{}", DEFAULT_INTERNAL_STUB, default_ext)];

        Self { external, internal, }
    }
}

#[derive(Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct ConfigRepr {
    #[serde(rename = "filtering")]
    pub selection_repr: SelectionRepr,
    #[serde(rename = "ordering")]
    pub sorter_repr: Sorter,
    #[serde(rename = "sourcing")]
    pub sources_repr: SourcesRepr,
}

#[derive(Deserialize)]
#[serde(try_from = "ConfigRepr")]
pub struct Config {
    pub selection: Selection,
    pub sorter: Sorter,
    pub sourcer: Sourcer,
}

impl TryFrom<ConfigRepr> for Config {
    type Error = Error;

    fn try_from(value: ConfigRepr) -> Result<Self, Self::Error> {
        let mut sources = Vec::new();

        let mut selection_repr = value.selection_repr;

        for name in value.sources_repr.external {
            let src = Source::from_name(name, Anchor::External)?;
            sources.push(src);
        }

        for name in value.sources_repr.internal {
            let src = Source::from_name(name, Anchor::Internal)?;
            sources.push(src);
        }

        if selection_repr.exclude_sources {
            // Add sources to the list of excluded files.
            for source in sources.iter() {
                let pattern = &source.name;
                selection_repr.exclude_files.add_pattern(pattern).map_err(Into::<MatcherError>::into)?;
            }
        }

        // Manually convert `SelectionRepr` into `Selection`.
        let selection = selection_repr.try_into()?;

        let sourcer = sources.into();

        Ok(Self {
            selection,
            sorter: value.sorter_repr,
            sourcer,
        })
    }
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
            config.sourcer.as_sources(),
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
            config.sourcer.as_sources(),
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
            config.sourcer.as_sources(),
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
            [sourcing]
            track = ["item_meta.yml"]
        "#;

        let config: Config = toml::from_str(&text_config).unwrap();

        assert_eq!(config.selection.is_file_pattern_match(&"music.flac"), true);
        assert_eq!(config.selection.is_file_pattern_match(&"music.mp3"), false);
        assert_eq!(config.selection.is_file_pattern_match(&"photo.png"), true);
        assert_eq!(config.sorter.sort_by, SortBy::Name);
        assert_eq!(
            config.sourcer.as_sources(),
            vec![
                Source::from_name(str!("item_meta.yml"), Anchor::External).unwrap(),
                Source::from_name(str!("album.json"), Anchor::Internal).unwrap(),
            ]
        );
    }
}
