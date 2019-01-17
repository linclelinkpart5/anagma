//! Types for modeling and representing item metadata.

pub mod key;
pub mod val;
pub(crate) mod repr;

use std::collections::BTreeMap;
use std::collections::HashMap;

pub use metadata::types::val::MetaVal;
pub use metadata::types::key::MetaKey;

pub type MetaBlock = BTreeMap<String, MetaVal>;
pub type MetaBlockSeq = Vec<MetaBlock>;
pub type MetaBlockMap = HashMap<String, MetaBlock>;

/// A data structure-level representation of all possible metadata types and their formats.
/// This is intended to be independent of the text-level representation of the metadata.
#[derive(Debug, Clone)]
pub enum MetaStructure {
    One(MetaBlock),
    Seq(MetaBlockSeq),
    Map(MetaBlockMap),
}
