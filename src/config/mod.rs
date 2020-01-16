//! Provides configuration options for a library, both programmatically and via config files.

pub mod meta_format;
pub mod selection;
pub mod sorter;

use self::meta_format::MetaFormat;
use self::selection::Selection;
use self::sorter::Sorter;

use crate::metadata::target::Target;

#[derive(Deserialize)]
#[serde(default)]
pub struct Config {
    #[serde(flatten)] pub selection: Selection,
    #[serde(flatten)] pub sorter: Sorter,
    pub item_fn: String,
    pub self_fn: String,
    pub meta_format: MetaFormat,
}

impl Default for Config {
    fn default() -> Self {
        // TODO: Is there a way to intelligently populate this while also
        //       preserving defaulting behavior?
        let selection = Selection::default();
        let sorter = Sorter::default();
        let meta_format = MetaFormat::default();
        let item_fn = format!(
            "{}.{}",
            Target::Siblings.default_file_name(),
            meta_format.file_extension(),
        );
        let self_fn = format!(
            "{}.{}",
            Target::Parent.default_file_name(),
            meta_format.file_extension(),
        );

        Self {
            selection,
            sorter,
            item_fn,
            self_fn,
            meta_format,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_yaml;

    use super::*;

    use super::sorter::sort_by::SortBy;

    #[test]
    fn test_deserialization() {
        let text_config = r#"
            include_files: '*.flac'
            sort_by: name
        "#;

        let config: Config = serde_yaml::from_str(&text_config).unwrap();

        assert_eq!(config.selection.is_file_pattern_match("music.flac"), true);
        assert_eq!(config.selection.is_file_pattern_match("music.mp3"), false);
        assert_eq!(config.selection.is_file_pattern_match("photo.png"), false);
        assert_eq!(config.selection.is_file_pattern_match("self.yml"), false);
        assert_eq!(config.selection.is_file_pattern_match("item.yml"), false);
        assert_eq!(config.sorter.sort_by, SortBy::Name);
        assert_eq!(config.item_fn, "item.yml");
        assert_eq!(config.self_fn, "self.yml");
        assert_eq!(config.meta_format, MetaFormat::Yaml);

        let text_config = r#"
            include_files:
                - '*.flac'
                - '*.mp3'
            sort_by: mod_time
        "#;

        let config: Config = serde_yaml::from_str(&text_config).unwrap();

        assert_eq!(config.selection.is_file_pattern_match("music.flac"), true);
        assert_eq!(config.selection.is_file_pattern_match("music.mp3"), true);
        assert_eq!(config.selection.is_file_pattern_match("photo.png"), false);
        assert_eq!(config.sorter.sort_by, SortBy::ModTime);
        assert_eq!(config.item_fn, "item.yml");
        assert_eq!(config.self_fn, "self.yml");
        assert_eq!(config.meta_format, MetaFormat::Yaml);

        let text_config = r#"
            include_files: '*'
            sort_by: mod_time
        "#;

        let config: Config = serde_yaml::from_str(&text_config).unwrap();

        assert_eq!(config.selection.is_file_pattern_match("music.flac"), true);
        assert_eq!(config.selection.is_file_pattern_match("music.mp3"), true);
        assert_eq!(config.selection.is_file_pattern_match("photo.png"), true);
        assert_eq!(config.sorter.sort_by, SortBy::ModTime);
        assert_eq!(config.item_fn, "item.yml");
        assert_eq!(config.self_fn, "self.yml");
        assert_eq!(config.meta_format, MetaFormat::Yaml);

        let text_config = r#"
            include_files: '*'
            exclude_files: '*.mp3'
            sort_by: name
            item_fn: item_meta.yml
            meta_format: yaml
        "#;

        let config: Config = serde_yaml::from_str(&text_config).unwrap();

        assert_eq!(config.selection.is_file_pattern_match("music.flac"), true);
        assert_eq!(config.selection.is_file_pattern_match("music.mp3"), false);
        assert_eq!(config.selection.is_file_pattern_match("photo.png"), true);
        assert_eq!(config.sorter.sort_by, SortBy::Name);
        assert_eq!(config.item_fn, "item_meta.yml");
        assert_eq!(config.self_fn, "self.yml");
        assert_eq!(config.meta_format, MetaFormat::Yaml);
    }
}
