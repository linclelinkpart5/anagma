mod inherit;
mod collect;

use std::path::Path;
use std::path::PathBuf;
use std::collections::VecDeque;
use std::collections::HashMap;

use config::meta_format::MetaFormat;
use config::selection::Selection;
use config::sort_order::SortOrder;
use metadata::types::MetaBlock;
use metadata::types::MetaKey;
use metadata::processor::MetaProcessor;

use self::inherit::InheritMethod;
use self::collect::CollectMethod;


pub type FallbackSpec = HashMap<String, FallbackSpecNode>;

/// Node type for the tree representation of fallback methods.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum FallbackSpecNode {
    Leaf(FallbackMethod),
    Pass(HashMap<String, FallbackSpecNode>),
    Both(FallbackMethod, HashMap<String, FallbackSpecNode>),
}

/// Different ways to process parent metadata into desired outputs.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
pub enum FallbackMethod {
    None,
    Inherit(InheritMethod),
    Collect(CollectMethod),
}

impl Default for FallbackMethod {
    fn default() -> Self {
        Self::Inherit(InheritMethod::default())
    }
}
