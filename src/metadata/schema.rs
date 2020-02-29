//! Data representations of meta files.

use serde::Deserialize;

use crate::metadata::block::Block;
use crate::metadata::block::BlockSequence;
use crate::metadata::block::BlockMapping;

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
#[derive(Debug, Clone)]
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
