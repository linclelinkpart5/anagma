//! Types for modeling and representing metadata for one or more items.

use indexmap::IndexMap;

use crate::metadata::value::ValueMapping;

/// Represents a chunk of metadata for one item.
// NOTE: This alias is intentional, this is taken advantage of later downstream.
pub type Block = ValueMapping;

/// Represents multiple chunks of metadata for an ordered collection of items.
pub type BlockSeq = Vec<Block>;

/// Represents multiple chunks of metadata for a mapping of items keyed by name.
pub type BlockMap = IndexMap<String, Block>;
