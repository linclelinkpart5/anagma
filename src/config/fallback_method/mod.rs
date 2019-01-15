mod inherit;
mod collect;

use std::collections::HashMap;

pub use self::inherit::InheritMethod;
pub use self::collect::CollectMethod;


pub type FallbackSpec = HashMap<String, FallbackSpecNode>;

/// Node type for the tree representation of fallback methods.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
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
