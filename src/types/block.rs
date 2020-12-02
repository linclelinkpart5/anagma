use std::collections::BTreeMap;

use crate::types::Value;

/// Represents a chunk of metadata for one item.
pub struct Block(BTreeMap<String, Value>);
