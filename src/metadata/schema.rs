//! Data representations of meta files.

use serde::{Deserialize, Serialize};
use strum::EnumDiscriminants;

use crate::source::Anchor;
use crate::types::{Block, BlockSeq, BlockMap};

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub(crate) enum UnitSchemaRepr {
    One(Block),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub(crate) enum ManySchemaRepr {
    Seq(BlockSeq),
    Map(BlockMap),
}

/// An easy-to-deserialize flavor of a meta structure.
/// The number of item files ("degree") a schema provides data for.
/// In other words, whether a schema provides data for one or many items.
#[derive(Debug, Clone, Deserialize, EnumDiscriminants)]
#[serde(untagged)]
#[strum_discriminants(name(Arity), vis(pub), derive(Hash))]
pub(crate) enum SchemaRepr {
    Unit(UnitSchemaRepr),
    Many(ManySchemaRepr),
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
    Seq(BlockSeq),
    Map(BlockMap),
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
