//! Types for modeling and representing item metadata.

pub mod key;
pub mod val;

use std::collections::BTreeMap;
use std::collections::HashMap;

pub use crate::metadata::types::val::MetaVal;
pub use crate::metadata::types::key::MetaKey;
pub use crate::metadata::types::key::MetaKeyPath;

pub type MetaBlock<'k> = BTreeMap<MetaKey<'k>, MetaVal<'k>>;
pub type MetaBlockSeq<'k> = Vec<MetaBlock<'k>>;
pub type MetaBlockMap<'k> = HashMap<String, MetaBlock<'k>>;

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum UnitMetaStructureRepr<'k> {
    One(MetaBlock<'k>),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum ManyMetaStructureRepr<'k> {
    Seq(MetaBlockSeq<'k>),
    Map(MetaBlockMap<'k>),
}

/// An easy-to-deserialize flavor of a meta structure.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum MetaStructureRepr<'k> {
    Unit(UnitMetaStructureRepr<'k>),
    Many(ManyMetaStructureRepr<'k>),
}

/// A data structure-level representation of all possible metadata types and their formats.
/// This is intended to be independent of the text-level representation of the metadata.
#[derive(Debug, Clone)]
pub enum MetaStructure<'k> {
    One(MetaBlock<'k>),
    Seq(MetaBlockSeq<'k>),
    Map(MetaBlockMap<'k>),
}

impl<'k> From<MetaStructureRepr<'k>> for MetaStructure<'k> {
    fn from(msr: MetaStructureRepr<'k>) -> Self {
        match msr {
            MetaStructureRepr::Unit(UnitMetaStructureRepr::One(mb)) => Self::One(mb),
            MetaStructureRepr::Many(ManyMetaStructureRepr::Seq(mb_seq)) => Self::Seq(mb_seq),
            MetaStructureRepr::Many(ManyMetaStructureRepr::Map(mb_map)) => Self::Map(mb_map),
        }
    }
}
