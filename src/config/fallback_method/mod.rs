//! Representation and processing logic for metadata fallbacks.
//! A fallback is a way to obtain metadata from another source if it is missing for an item.

use std::collections::HashMap;

use metadata::types::MetaKey;

/// Fallbacks that source missing data by looking at the ancestors/parents of an item.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InheritFallback {
    Override,
    Merge,
}

impl Default for InheritFallback {
    fn default() -> Self {
        Self::Override
    }
}

/// Fallbacks that source missing data by looking at the descendants/children of an item.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HarvestFallback {
    Collect,
    First,
}

impl Default for HarvestFallback {
    fn default() -> Self {
        Self::Collect
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
pub enum Fallback {
    Inherit(InheritFallback),
    Harvest(HarvestFallback),
}

impl Default for Fallback {
    fn default() -> Self {
        Self::Inherit(InheritFallback::default())
    }
}

/// Node type for the tree representation of fallback methods.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum FallbackSpecNode {
    Leaf(Option<Fallback>),
    Pass(HashMap<MetaKey, FallbackSpecNode>),
    Both(Option<Fallback>, HashMap<MetaKey, FallbackSpecNode>),
}

pub type FallbackSpec = HashMap<MetaKey, FallbackSpecNode>;

#[cfg(test)]
mod tests {}
