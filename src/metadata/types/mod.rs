//! Types for modeling and representing item metadata.

pub mod key;
pub mod val;

use std::collections::BTreeMap;
use std::collections::HashMap;

use metadata::types::val::MetaVal;

pub type MetaBlock = BTreeMap<String, MetaVal>;
pub type MetaBlockSeq = Vec<MetaBlock>;
pub type MetaBlockMap = HashMap<String, MetaBlock>;
