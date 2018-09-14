//! Types for modeling and representing item metadata.

pub mod key;
pub mod val;

use std::collections::BTreeMap;
use std::collections::HashMap;

use metadata::types::val::MetaValue;

pub type MetaBlock = BTreeMap<String, MetaValue>;
pub type MetaBlockSeq = Vec<MetaBlock>;
pub type MetaBlockMap = HashMap<String, MetaBlock>;
