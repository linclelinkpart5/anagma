//! Data representations of meta files.

use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

use crate::types::Value;

/// A metadata block, consisting of key-value pairs (aka "fields").
pub type Block = BTreeMap<String, Value>;

/// Represents a collection of metadata blocks.
/// Metadata blocks may be untagged, or tagged with a file name.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Blocks {
    Untagged(Vec<Block>),
    Tagged(HashMap<String, Block>),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Metadata {
    album: Block,
    tracks: Blocks,
}
