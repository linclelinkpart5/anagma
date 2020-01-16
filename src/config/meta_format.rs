//! Defines the format of metadata files to be parsed.

use std::path::Path;
use std::fs::File;
use std::io::Read;

use crate::metadata::target::Target;
use crate::metadata::structure::MetaStructure;
use crate::metadata::structure::MetaStructureRepr;

#[derive(Debug)]
pub enum Error {
    CannotOpenFile(std::io::Error),
    CannotReadFile(std::io::Error),
    YamlDeserializeError(serde_yaml::Error),
    JsonDeserializeError(serde_json::Error),
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

/// Represents all the different metadata formats that are supported.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetaFormat {
    Yaml,
    Json,
}

impl Default for MetaFormat {
    fn default() -> Self {
        Self::Yaml
    }
}

impl MetaFormat {
    /// Returns the expected file name extension for files in this format.
    // TODO: When `match` is allowed in `const fn`s, make this `const fn`.
    pub fn file_extension(&self) -> &'static str {
        match self {
            Self::Yaml => "yml",
            Self::Json => "json",
        }
    }

    fn from_yaml_str(s: &str, mt: Target) -> Result<MetaStructure, Error> {
        match mt {
            Target::Parent => {
                serde_yaml::from_str(s)
                .map_err(Error::YamlDeserializeError)
                .map(MetaStructureRepr::Unit)
            },
            Target::Siblings => {
                serde_yaml::from_str(s)
                .map_err(Error::YamlDeserializeError)
                .map(MetaStructureRepr::Many)
            },
        }.map(Into::into)
    }

    fn from_json_str(s: &str, mt: Target) -> Result<MetaStructure, Error> {
        match mt {
            Target::Parent => {
                serde_json::from_str(s)
                .map_err(Error::JsonDeserializeError)
                .map(MetaStructureRepr::Unit)
            },
            Target::Siblings => {
                serde_json::from_str(s)
                .map_err(Error::JsonDeserializeError)
                .map(MetaStructureRepr::Many)
            },
        }.map(Into::into)
    }

    pub fn from_str(&self, s: &str, mt: Target) -> Result<MetaStructure, Error> {
        match self {
            Self::Yaml => Self::from_yaml_str(s, mt),
            Self::Json => Self::from_json_str(s, mt),
        }
    }

    pub fn from_file<P: AsRef<Path>>(&self, p: P, mt: Target) -> Result<MetaStructure, Error> {
        let p = p.as_ref();
        let mut f = File::open(p).map_err(Error::CannotOpenFile)?;

        let mut buffer = String::new();
        f.read_to_string(&mut buffer).map_err(Error::CannotReadFile)?;

        self.from_str(&buffer, mt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_yaml_str() {
        let input = r#"
            key_a: val_a
            key_b: val_b
            key_c: val_c
            key_d: val_d
        "#;
        assert_matches!(MetaFormat::from_yaml_str(input, Target::Parent), Ok(MetaStructure::One(_)));

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
        assert_matches!(MetaFormat::from_yaml_str(input, Target::Parent), Ok(MetaStructure::One(_)));

        let input = r#"
            -   key_1_a: val_1_a
                key_1_b: val_1_b
            -   key_2_a: val_2_a
                key_2_b: val_2_b
        "#;
        assert_matches!(MetaFormat::from_yaml_str(input, Target::Siblings), Ok(MetaStructure::Seq(_)));

        let input = r#"
            item_1:
                key_1_a: val_1_a
                key_1_b: val_1_b
            item_2:
                key_2_a: val_2_a
                key_2_b: val_2_b
        "#;
        assert_matches!(MetaFormat::from_yaml_str(input, Target::Siblings), Ok(MetaStructure::Map(_)));
    }

    #[test]
    fn test_from_json_str() {
        let input = r#"
        {
            "key_a": "val_a",
            "key_b": "val_b",
            "key_c": "val_c",
            "key_d": "val_d"
        }
        "#;
        assert_matches!(MetaFormat::from_json_str(input, Target::Parent), Ok(MetaStructure::One(_)));

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
        assert_matches!(MetaFormat::from_json_str(input, Target::Parent), Ok(MetaStructure::One(_)));

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
        assert_matches!(MetaFormat::from_json_str(input, Target::Siblings), Ok(MetaStructure::Seq(_)));

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
        assert_matches!(MetaFormat::from_json_str(input, Target::Siblings), Ok(MetaStructure::Map(_)));
    }
}
