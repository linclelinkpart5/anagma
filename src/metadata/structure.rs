use crate::metadata::block::MetaBlock;
use crate::metadata::block::MetaBlockSeq;
use crate::metadata::block::MetaBlockMap;

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum UnitMetaStructureRepr {
    One(MetaBlock),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum ManyMetaStructureRepr {
    Seq(MetaBlockSeq),
    Map(MetaBlockMap),
}

/// An easy-to-deserialize flavor of a meta structure.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum MetaStructureRepr {
    Unit(UnitMetaStructureRepr),
    Many(ManyMetaStructureRepr),
}

/// A data structure-level representation of all possible metadata types and their formats.
/// This is intended to be independent of the text-level representation of the metadata.
#[derive(Debug, Clone)]
pub enum MetaStructure {
    One(MetaBlock),
    Seq(MetaBlockSeq),
    Map(MetaBlockMap),
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
