//! Data representations of meta files.

use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::io::Error as IoError;

use serde::Deserialize;
use serde::Serialize;
use serde_yaml::Error as YamlError;
use serde_json::Error as JsonError;
use strum::EnumString;
use thiserror::Error;

use crate::metadata::block::Block;
use crate::metadata::block::BlockSequence;
use crate::metadata::block::BlockMapping;
use crate::source::Anchor;

#[derive(Debug, Error)]
pub enum Error {
    #[error("cannot open metadata file: {0}")]
    CannotOpenFile(#[source] IoError),
    #[error("cannot read metadata file: {0}")]
    CannotReadFile(#[source] IoError),
    #[error("cannot deserialize YAML: {0}")]
    YamlDeserialize(#[source] YamlError),
    #[error("cannot deserialize JSON: {0}")]
    JsonDeserialize(#[source] JsonError),
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

/// The "degree" of the number of item files a schema provides data for.
/// In other words, whether a schema provides data for one or many items.
// TODO: Replace with `EnumDiscriminants` once PR to `strum` is merged/released.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Arity {
    Unit,
    Many,
}

impl From<Anchor> for Arity {
    fn from(value: Anchor) -> Self {
        match value {
            Anchor::Internal => Arity::Unit,
            Anchor::External => Arity::Many,
        }
    }
}

impl<'a> From<&'a Anchor> for &'a Arity {
    fn from(value: &'a Anchor) -> Self {
        match value {
            Anchor::Internal => &Arity::Unit,
            Anchor::External => &Arity::Many,
        }
    }
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
    pub fn expects_sorted(&self) -> bool {
        match self {
            Self::One(..) => false,
            Self::Seq(..) => true,
            Self::Map(..) => false,
        }
    }

    fn from_yaml_str(s: &str, arity: &Arity) -> Result<Self, YamlError> {
        match arity {
            Arity::Unit => serde_yaml::from_str(s).map(SchemaRepr::Unit),
            Arity::Many => serde_yaml::from_str(s).map(SchemaRepr::Many),
        }.map(Into::into)
    }

    fn from_json_str(s: &str, arity: &Arity) -> Result<Self, JsonError> {
        match arity {
            Arity::Unit => serde_json::from_str(s).map(SchemaRepr::Unit),
            Arity::Many => serde_json::from_str(s).map(SchemaRepr::Many),
        }.map(Into::into)
    }

    pub fn from_str(format: &SchemaFormat, s: &str, arity: &Arity) -> Result<Schema, Error> {
        match format {
            SchemaFormat::Yaml => Self::from_yaml_str(s, arity).map_err(Error::YamlDeserialize),
            SchemaFormat::Json => Self::from_json_str(s, arity).map_err(Error::JsonDeserialize),
        }
    }

    pub fn from_file(format: &SchemaFormat, path: &Path, arity: &Arity) -> Result<Schema, Error> {
        let mut f = File::open(path).map_err(Error::CannotOpenFile)?;

        let mut buffer = String::new();
        f.read_to_string(&mut buffer).map_err(Error::CannotReadFile)?;

        Self::from_str(format, &buffer, arity)
    }
}

/// Represents all the different metadata formats that are supported.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Deserialize, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum SchemaFormat {
    #[strum(serialize = "json", serialize = "JSON")]
    Json,
    #[strum(serialize = "yml", serialize = "YML")]
    Yaml,
}

impl Default for SchemaFormat {
    fn default() -> Self {
        Self::Json
    }
}

impl SchemaFormat {
    /// Returns the expected file name extension for files in this format.
    pub const fn file_extension(&self) -> &'static str {
        match self {
            Self::Yaml => "yml",
            Self::Json => "json",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_yaml_str() {
        let input = r#"
            key_a: val_a
            key_b: val_b
            key_c: val_c
            key_d: val_d
        "#;
        assert!(matches!(Schema::from_yaml_str(input, &Arity::Unit), Ok(Schema::One(_))));

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
        assert!(matches!(Schema::from_yaml_str(input, &Arity::Unit), Ok(Schema::One(_))));

        let input = r#"
            -   key_1_a: val_1_a
                key_1_b: val_1_b
            -   key_2_a: val_2_a
                key_2_b: val_2_b
        "#;
        assert!(matches!(Schema::from_yaml_str(input, &Arity::Many), Ok(Schema::Seq(_))));

        let input = r#"
            item_1:
                key_1_a: val_1_a
                key_1_b: val_1_b
            item_2:
                key_2_a: val_2_a
                key_2_b: val_2_b
        "#;
        assert!(matches!(Schema::from_yaml_str(input, &Arity::Many), Ok(Schema::Map(_))));
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
        assert!(matches!(Schema::from_json_str(input, &Arity::Unit), Ok(Schema::One(_))));

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
        assert!(matches!(Schema::from_json_str(input, &Arity::Unit), Ok(Schema::One(_))));

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
        assert!(matches!(Schema::from_json_str(input, &Arity::Many), Ok(Schema::Seq(_))));

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
        assert!(matches!(Schema::from_json_str(input, &Arity::Many), Ok(Schema::Map(_))));
    }
}
