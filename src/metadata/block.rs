//! Types for modeling and representing item metadata.

use std::collections::BTreeMap;

use indexmap::IndexMap;

use crate::metadata::value::Value;

pub type Block = BTreeMap<String, Value>;
pub type BlockSeq = Vec<Block>;
pub type BlockMap = IndexMap<String, Block>;
