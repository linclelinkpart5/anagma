mod inherit;
mod collect;

use std::path::Path;
use std::collections::HashMap;

use config::meta_format::MetaFormat;
use config::selection::Selection;
use config::sort_order::SortOrder;
use metadata::types::MetaBlock;
use metadata::processor::MetaProcessor;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
pub enum SuperFallback {
    // Traverses ancestors.
    Override,
    Merge,

    // Traverses children.
    Collect,
    First,

    // No traversing.
    None,
}

impl Default for SuperFallback {
    fn default() -> Self {
        Self::Override
    }
}

impl SuperFallback {
    pub fn travserses_ancestors(&self) -> bool {
        match self {
            Self::Override => true,
            Self::Merge => true,
            Self::Collect => false,
            Self::First => false,
            Self::None => false,
        }
    }

    pub fn travserses_children(&self) -> bool {
        match self {
            Self::Override => false,
            Self::Merge => false,
            Self::Collect => true,
            Self::First => true,
            Self::None => false,
        }
    }
}

/// Different ways to process parent metadata into desired outputs.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AncestorFallback {
    Override,
    Merge,
}

impl Default for AncestorFallback {
    fn default() -> Self {
        Self::Override
    }
}

/// Different ways to process child metadata into desired outputs.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChildrenFallback {
    Collect,
    First,
}

impl Default for ChildrenFallback {
    fn default() -> Self {
        Self::Collect
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
pub enum Fallback {
    None,
    Ancestor(AncestorFallback),
    Children(ChildrenFallback),
}

impl Default for Fallback {
    fn default() -> Self {
        Self::Ancestor(AncestorFallback::default())
    }
}

pub type AncestorFallbackSpec = HashMap<String, AncestorFallbackSpecNode>;

/// Node type for the tree representation of fallback methods.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum AncestorFallbackSpecNode {
    Leaf(AncestorFallback),
    Pass(HashMap<String, AncestorFallbackSpecNode>),
    Both(AncestorFallback, HashMap<String, AncestorFallbackSpecNode>),
}

pub type ChildrenFallbackSpec = HashMap<String, ChildrenFallbackSpecNode>;

/// Node type for the tree representation of fallback methods.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ChildrenFallbackSpecNode {
    Leaf(ChildrenFallback),
    Pass(HashMap<String, ChildrenFallbackSpecNode>),
    Both(ChildrenFallback, HashMap<String, ChildrenFallbackSpecNode>),
}

pub type FallbackSpec = HashMap<String, FallbackSpecNode>;

/// Node type for the tree representation of fallback methods.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum FallbackSpecNode {
    Leaf(Fallback),
    Pass(HashMap<String, FallbackSpecNode>),
    Both(Fallback, HashMap<String, FallbackSpecNode>),
}


fn divide(mut fallback_spec: FallbackSpec) -> (AncestorFallbackSpec, ChildrenFallbackSpec) {
    let mut a_fs = AncestorFallbackSpec::new();
    let mut c_fs = ChildrenFallbackSpec::new();

    for (k, fsn) in fallback_spec.drain() {
        // Check what kind of node this is.
        match fsn {
            FallbackSpecNode::Leaf(f) => {
            },
            FallbackSpecNode::Pass(map) => {},
            FallbackSpecNode::Both(f, map) => {},
        }
    }

    (a_fs, c_fs)
}

fn divide_node(mut fallback_spec_node: FallbackSpecNode) -> () {

}

pub fn process_fallbacks<P: AsRef<Path>>(
    start_item_path: P,
    meta_format: MetaFormat,
    selection: &Selection,
    sort_order: SortOrder,
    fallback_spec: &FallbackSpec,
    default_fallback: Fallback,
) -> MetaBlock
{
    // Load the origin metadata.
    // This is the isolated metadata block for just the starting item.
    let origin_mb = MetaProcessor::process_item_file(
        start_item_path,
        meta_format,
        selection,
        sort_order,
    ).unwrap();

    MetaBlock::new()
}
