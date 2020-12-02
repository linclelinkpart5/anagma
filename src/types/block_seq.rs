use crate::types::Block;

/// Represents multiple chunks of metadata for an ordered collection of items.
pub struct BlockSeq(Vec<Block>);
