use std::path::Path;
use std::fs::File;
use std::io::{Error as IoError, Read};

use serde::Deserialize;
use serde_yaml::Error as YamlError;
use serde_json::Error as JsonError;
use strum::{EnumString, EnumIter, AsRefStr};
use thiserror::Error;

use crate::metadata::{Arity, Schema, SchemaRepr};

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

/// Represents all the different metadata formats that are supported.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Deserialize, EnumString, EnumIter, AsRefStr)]
#[strum(serialize_all = "snake_case")]
pub enum Format {
    #[strum(serialize = "JSON", serialize = "json")]
    Json,
    #[strum(serialize = "YML", serialize = "yml")]
    Yaml,
}

impl Format {
    fn read_yaml(s: &str, arity: &Arity) -> Result<Schema, YamlError> {
        match arity {
            Arity::Unit => serde_yaml::from_str(s).map(SchemaRepr::Unit),
            Arity::Many => serde_yaml::from_str(s).map(SchemaRepr::Many),
        }.map(Into::into)
    }

    fn read_json(s: &str, arity: &Arity) -> Result<Schema, JsonError> {
        match arity {
            Arity::Unit => serde_json::from_str(s).map(SchemaRepr::Unit),
            Arity::Many => serde_json::from_str(s).map(SchemaRepr::Many),
        }.map(Into::into)
    }

    pub fn read_schema_str(&self, s: &str, arity: &Arity) -> Result<Schema, Error> {
        match self {
            Self::Yaml => Self::read_yaml(s, arity).map_err(Error::YamlDeserialize),
            Self::Json => Self::read_json(s, arity).map_err(Error::JsonDeserialize),
        }
    }

    pub fn read_schema_path(&self, path: &Path, arity: &Arity) -> Result<Schema, Error> {
        let mut f = File::open(path).map_err(Error::CannotOpenFile)?;

        let mut buffer = String::new();
        f.read_to_string(&mut buffer).map_err(Error::CannotReadFile)?;

        self.read_schema_str(&buffer, arity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_yaml() {
        let input = r#"
            key_a: val_a
            key_b: val_b
            key_c: val_c
            key_d: val_d
        "#;
        assert!(matches!(Format::read_yaml(input, &Arity::Unit), Ok(Schema::One(_))));

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
        assert!(matches!(Format::read_yaml(input, &Arity::Unit), Ok(Schema::One(_))));

        let input = r#"
            -   key_1_a: val_1_a
                key_1_b: val_1_b
            -   key_2_a: val_2_a
                key_2_b: val_2_b
        "#;
        assert!(matches!(Format::read_yaml(input, &Arity::Many), Ok(Schema::Seq(_))));

        let input = r#"
            item_1:
                key_1_a: val_1_a
                key_1_b: val_1_b
            item_2:
                key_2_a: val_2_a
                key_2_b: val_2_b
        "#;
        assert!(matches!(Format::read_yaml(input, &Arity::Many), Ok(Schema::Map(_))));
    }

    #[test]
    fn read_json() {
        let input = r#"
        {
            "key_a": "val_a",
            "key_b": "val_b",
            "key_c": "val_c",
            "key_d": "val_d"
        }
        "#;
        assert!(matches!(Format::read_json(input, &Arity::Unit), Ok(Schema::One(_))));

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
        assert!(matches!(Format::read_json(input, &Arity::Unit), Ok(Schema::One(_))));

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
        assert!(matches!(Format::read_json(input, &Arity::Many), Ok(Schema::Seq(_))));

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
        assert!(matches!(Format::read_json(input, &Arity::Many), Ok(Schema::Map(_))));
    }
}
