//! Provides configuration options for a library, both programmatically and via config files.

pub mod selection;
pub mod sorter;

use std::path::Path;

use serde::Deserialize;

use self::selection::Selection;
use self::sorter::Sorter;

use crate::metadata::schema::SchemaFormat;

#[derive(Deserialize)]
#[serde(default)]
pub struct Config {
    #[serde(flatten)] pub selection: Selection,
    #[serde(flatten)] pub sorter: Sorter,
    pub item_fn: String,
    pub self_fn: String,
    pub schema_format: SchemaFormat,
}

impl Default for Config {
    fn default() -> Self {
        let selection = Selection::default();
        let sorter = Sorter::default();
        let schema_format = SchemaFormat::default();
        let item_fn = format!(
            "item.{}",
            schema_format.file_extension(),
        );
        let self_fn = format!(
            "self.{}",
            schema_format.file_extension(),
        );

        Self {
            selection,
            sorter,
            item_fn,
            self_fn,
            schema_format,
        }
    }
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: &P) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(path)?;
        let config = serde_json::from_str(&contents)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use serde_yaml;

    use super::*;

    use super::sorter::sort_by::SortBy;

    #[test]
    fn deserialization() {
        let text_config = r#"
            include_files: '*.flac'
            sort_by: name
        "#;

        let config: Config = serde_yaml::from_str(&text_config).unwrap();

        assert_eq!(config.selection.is_file_pattern_match(&"music.flac"), true);
        assert_eq!(config.selection.is_file_pattern_match(&"music.mp3"), false);
        assert_eq!(config.selection.is_file_pattern_match(&"photo.png"), false);
        assert_eq!(config.selection.is_file_pattern_match(&"self.yml"), false);
        assert_eq!(config.selection.is_file_pattern_match(&"item.yml"), false);
        assert_eq!(config.sorter.sort_by, SortBy::Name);
        assert_eq!(config.item_fn, "item.json");
        assert_eq!(config.self_fn, "self.json");
        assert_eq!(config.schema_format, SchemaFormat::Json);

        let text_config = r#"
            include_files:
                - '*.flac'
                - '*.mp3'
            sort_by: mod_time
        "#;

        let config: Config = serde_yaml::from_str(&text_config).unwrap();

        assert_eq!(config.selection.is_file_pattern_match(&"music.flac"), true);
        assert_eq!(config.selection.is_file_pattern_match(&"music.mp3"), true);
        assert_eq!(config.selection.is_file_pattern_match(&"photo.png"), false);
        assert_eq!(config.sorter.sort_by, SortBy::ModTime);
        assert_eq!(config.item_fn, "item.json");
        assert_eq!(config.self_fn, "self.json");
        assert_eq!(config.schema_format, SchemaFormat::Json);

        let text_config = r#"
            include_files: '*'
            sort_by: mod_time
        "#;

        let config: Config = serde_yaml::from_str(&text_config).unwrap();

        assert_eq!(config.selection.is_file_pattern_match(&"music.flac"), true);
        assert_eq!(config.selection.is_file_pattern_match(&"music.mp3"), true);
        assert_eq!(config.selection.is_file_pattern_match(&"photo.png"), true);
        assert_eq!(config.sorter.sort_by, SortBy::ModTime);
        assert_eq!(config.item_fn, "item.json");
        assert_eq!(config.self_fn, "self.json");
        assert_eq!(config.schema_format, SchemaFormat::Json);

        let text_config = r#"
            include_files: '*'
            exclude_files: '*.mp3'
            sort_by: name
            item_fn: item_meta.yml
            schema_format: yaml
        "#;

        let config: Config = serde_yaml::from_str(&text_config).unwrap();

        assert_eq!(config.selection.is_file_pattern_match(&"music.flac"), true);
        assert_eq!(config.selection.is_file_pattern_match(&"music.mp3"), false);
        assert_eq!(config.selection.is_file_pattern_match(&"photo.png"), true);
        assert_eq!(config.sorter.sort_by, SortBy::Name);
        assert_eq!(config.item_fn, "item_meta.yml");
        assert_eq!(config.self_fn, "self.json");
        assert_eq!(config.schema_format, SchemaFormat::Yaml);
    }
}
