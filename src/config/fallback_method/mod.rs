mod inherit;
mod collect;

use std::path::Path;
use std::collections::HashMap;

use config::meta_format::MetaFormat;
use config::selection::Selection;
use config::sort_order::SortOrder;
use metadata::types::MetaBlock;
use metadata::processor::MetaProcessor;

/// Different ways to process parent metadata into desired outputs.
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

/// Different ways to process child metadata into desired outputs.
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
    None,
}

impl Default for Fallback {
    fn default() -> Self {
        Self::Inherit(InheritFallback::default())
    }
}

/// Node type for the tree representation of fallback methods.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum FallbackSpecNode {
    Leaf(Fallback),
    Pass(HashMap<String, FallbackSpecNode>),
    Both(Fallback, HashMap<String, FallbackSpecNode>),
}

pub type FallbackSpec = HashMap<String, FallbackSpecNode>;


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
