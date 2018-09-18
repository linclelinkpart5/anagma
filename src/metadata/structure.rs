use metadata::types::MetaBlock;
use metadata::types::MetaBlockSeq;
use metadata::types::MetaBlockMap;

/// A data structure-level representation of all possible metadata types and their formats.
/// This is intended to be independent of the text-level representation of the metadata.
#[derive(Debug, Clone)]
pub enum MetaStructure {
    One(MetaBlock),
    Seq(MetaBlockSeq),
    Map(MetaBlockMap),
}
