//! Types for modeling and representing item metadata.

pub mod key;
pub mod val;

use std::collections::BTreeMap;
use std::collections::HashMap;
// use std::ffi::OsString;

pub use metadata::types::val::MetaVal;
pub use metadata::types::key::MetaKey;

pub type MetaBlock = BTreeMap<String, MetaVal>;
pub type MetaBlockSeq = Vec<MetaBlock>;
pub type MetaBlockMap = HashMap<String, MetaBlock>;
