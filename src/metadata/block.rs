//! Types for modeling and representing item metadata.

use std::collections::BTreeMap;

use indexmap::IndexMap;

use crate::metadata::value::Value;

pub type MetaBlock = BTreeMap<String, Value>;
pub type MetaBlockSeq = Vec<MetaBlock>;
pub type MetaBlockMap = IndexMap<String, MetaBlock>;
