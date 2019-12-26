//! Types for modeling and representing metadata for one or more items.

use indexmap::IndexMap;

use crate::metadata::value::Mapping;

/// Represents a chunk of metadata for one item.
// NOTE: This alias is intentional, this is taken advantage of later downstream.
pub type Block = Mapping;

/// Represents multiple chunks of metadata for an ordered collection of items.
pub type BlockSequence = Vec<Block>;

/// Represents multiple chunks of metadata for a mapping of items keyed by name.
pub type BlockMapping = IndexMap<String, Block>;
