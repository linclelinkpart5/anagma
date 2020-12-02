use indexmap::IndexMap;

use crate::types::Block;

/// Represents multiple chunks of metadata for a mapping of items keyed by name.
pub struct BlockMap(IndexMap<String, Block>);
