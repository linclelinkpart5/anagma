//! Data representations of meta files.

use std::path::Path;
use std::fs::File;
use std::io::Read;

use serde::Deserialize;
use serde::Serialize;
use serde_yaml::Error as YamlError;
use serde_json::Error as JsonError;

use crate::metadata::block::Block;
use crate::metadata::block::BlockSequence;
use crate::metadata::block::BlockMapping;
use crate::metadata::target::Target;

#[derive(Debug)]
pub enum Error {
    CannotOpenFile(std::io::Error),
    CannotReadFile(std::io::Error),
    YamlDeserializeError(YamlError),
    JsonDeserializeError(JsonError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::CannotOpenFile(ref err) =>
                write!(f, "cannot open metadata file: {}", err),
            Self::CannotReadFile(ref err) =>
                write!(f, "cannot read metadata file: {}", err),
            Self::YamlDeserializeError(ref err) =>
                write!(f, "cannot deserialize YAML: {}", err),
            Self::JsonDeserializeError(ref err) =>
                write!(f, "cannot deserialize JSON: {}", err),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::CannotOpenFile(ref err) => Some(err),
            Self::CannotReadFile(ref err) => Some(err),
            Self::YamlDeserializeError(ref err) => Some(err),
            Self::JsonDeserializeError(ref err) => Some(err),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub(crate) enum UnitSchemaRepr {
    One(Block),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub(crate) enum ManySchemaRepr {
    Seq(BlockSequence),
    Map(BlockMapping),
}

/// An easy-to-deserialize flavor of a meta structure.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub(crate) enum SchemaRepr {
    Unit(UnitSchemaRepr),
    Many(ManySchemaRepr),
}

/// A data structure-level representation of all metadata structures.
/// This is intended to be agnostic to the text-level format of the metadata.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum Schema {
    One(Block),
    Seq(BlockSequence),
    Map(BlockMapping),
}

impl From<SchemaRepr> for Schema {
    fn from(msr: SchemaRepr) -> Self {
        match msr {
            SchemaRepr::Unit(UnitSchemaRepr::One(mb)) => Self::One(mb),
            SchemaRepr::Many(ManySchemaRepr::Seq(mb_seq)) => Self::Seq(mb_seq),
            SchemaRepr::Many(ManySchemaRepr::Map(mb_map)) => Self::Map(mb_map),
        }
    }
}

impl Schema {
    fn from_yaml_str(s: &str, target: &Target) -> Result<Self, YamlError> {
        match target {
            Target::Parent => serde_yaml::from_str(s).map(SchemaRepr::Unit),
            Target::Siblings => serde_yaml::from_str(s).map(SchemaRepr::Many),
        }.map(Into::into)
    }

    fn from_json_str(s: &str, target: &Target) -> Result<Self, JsonError> {
        match target {
            Target::Parent => serde_json::from_str(s).map(SchemaRepr::Unit),
            Target::Siblings => serde_json::from_str(s).map(SchemaRepr::Many),
        }.map(Into::into)
    }

    pub fn from_str(format: &SchemaFormat, s: &str, target: &Target) -> Result<Schema, Error> {
        match format {
            SchemaFormat::Yaml => Self::from_yaml_str(s, target).map_err(Error::YamlDeserializeError),
            SchemaFormat::Json => Self::from_json_str(s, target).map_err(Error::JsonDeserializeError),
        }
    }

    pub fn from_file(format: &SchemaFormat, path: &Path, target: &Target) -> Result<Schema, Error> {
        let mut f = File::open(path).map_err(Error::CannotOpenFile)?;

        let mut buffer = String::new();
        f.read_to_string(&mut buffer).map_err(Error::CannotReadFile)?;

        Self::from_str(format, &buffer, target)
    }
}

/// Represents all the different metadata formats that are supported.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SchemaFormat {
    Yaml,
    Json,
}

impl Default for SchemaFormat {
    fn default() -> Self {
        Self::Yaml
    }
}

impl SchemaFormat {
    /// Returns the expected file name extension for files in this format.
    // TODO: When `match` is allowed in `const fn`s, make this `const fn`.
    pub fn file_extension(&self) -> &'static str {
        match self {
            Self::Yaml => "yml",
            Self::Json => "json",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use matches::assert_matches;

    #[test]
    fn from_yaml_str() {
        let input = r#"
            key_a: val_a
            key_b: val_b
            key_c: val_c
            key_d: val_d
        "#;
        assert_matches!(Schema::from_yaml_str(input, &Target::Parent), Ok(Schema::One(_)));

        let input = r#"
            key_a: val_a
            key_b:
                sub_key_a: sub_val_a
                sub_key_b: sub_val_b
            key_c: [val_a, val_b]
            key_d: {sub_key_a: sub_val_a, sub_key_b: sub_val_b}
            key_e:
                -   val_a
                -   val_b
        "#;
        assert_matches!(Schema::from_yaml_str(input, &Target::Parent), Ok(Schema::One(_)));

        let input = r#"
            -   key_1_a: val_1_a
                key_1_b: val_1_b
            -   key_2_a: val_2_a
                key_2_b: val_2_b
        "#;
        assert_matches!(Schema::from_yaml_str(input, &Target::Siblings), Ok(Schema::Seq(_)));

        let input = r#"
            item_1:
                key_1_a: val_1_a
                key_1_b: val_1_b
            item_2:
                key_2_a: val_2_a
                key_2_b: val_2_b
        "#;
        assert_matches!(Schema::from_yaml_str(input, &Target::Siblings), Ok(Schema::Map(_)));
    }

    #[test]
    fn from_json_str() {
        let input = r#"
        {
            "key_a": "val_a",
            "key_b": "val_b",
            "key_c": "val_c",
            "key_d": "val_d"
        }
        "#;
        assert_matches!(Schema::from_json_str(input, &Target::Parent), Ok(Schema::One(_)));

        let input = r#"
        {
            "key_a": "val_a",
            "key_b": {
                "sub_key_a": "sub_val_a",
                "sub_key_b": "sub_val_b"
            },
            "key_c": [
                "val_a",
                "val_b"
            ],
            "key_d": {
                "sub_key_a": "sub_val_a",
                "sub_key_b": "sub_val_b"
            },
            "key_e": [
                "val_a",
                "val_b"
            ]
        }
        "#;
        assert_matches!(Schema::from_json_str(input, &Target::Parent), Ok(Schema::One(_)));

        let input = r#"
        [
            {
                "key_1_a": "val_1_a",
                "key_1_b": "val_1_b"
            },
            {
                "key_2_a": "val_2_a",
                "key_2_b": "val_2_b"
            }
        ]
        "#;
        assert_matches!(Schema::from_json_str(input, &Target::Siblings), Ok(Schema::Seq(_)));

        let input = r#"
        {
            "item_1": {
                "key_1_a": "val_1_a",
                "key_1_b": "val_1_b"
            },
            "item_2": {
                "key_2_a": "val_2_a",
                "key_2_b": "val_2_b"
            }
        }
        "#;
        assert_matches!(Schema::from_json_str(input, &Target::Siblings), Ok(Schema::Map(_)));
    }
}
