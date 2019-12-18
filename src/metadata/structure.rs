use crate::metadata::block::Block;
use crate::metadata::block::BlockSeq;
use crate::metadata::block::BlockMap;

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub(crate) enum UnitMetaStructureRepr {
    One(Block),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub(crate) enum ManyMetaStructureRepr {
    Seq(BlockSeq),
    Map(BlockMap),
}

/// An easy-to-deserialize flavor of a meta structure.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub(crate) enum MetaStructureRepr {
    Unit(UnitMetaStructureRepr),
    Many(ManyMetaStructureRepr),
}

/// A data structure-level representation of all possible metadata types and their formats.
/// This is intended to be independent of the text-level representation of the metadata.
#[derive(Debug, Clone)]
pub enum MetaStructure {
    One(Block),
    Seq(BlockSeq),
    Map(BlockMap),
}

impl From<MetaStructureRepr> for MetaStructure {
    fn from(msr: MetaStructureRepr) -> Self {
        match msr {
            MetaStructureRepr::Unit(UnitMetaStructureRepr::One(mb)) => Self::One(mb),
            MetaStructureRepr::Many(ManyMetaStructureRepr::Seq(mb_seq)) => Self::Seq(mb_seq),
            MetaStructureRepr::Many(ManyMetaStructureRepr::Map(mb_map)) => Self::Map(mb_map),
        }
    }
}
