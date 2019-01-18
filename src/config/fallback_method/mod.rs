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
    Pass(FallbackSpec),
    Both(Fallback, FallbackSpec),
}

pub type FallbackSpec = HashMap<String, FallbackSpecNode>;

fn listify_fallback_spec(fallback_spec: &FallbackSpec) -> HashMap<Vec<&String>, Fallback> {
    let mut mapping = HashMap::new();

    listify_fallback_spec_helper(fallback_spec, vec![], &mut mapping);

    mapping
}

fn listify_fallback_spec_helper<'a>(
    fallback_spec: &'a FallbackSpec,
    curr_path: Vec<&'a String>,
    mapping: &mut HashMap<Vec<&'a String>, Fallback>)
{
    for (k, fsn) in fallback_spec {
        let mut new_path = curr_path.clone();
        new_path.push(k);

        match fsn {
            FallbackSpecNode::Leaf(fb) => {
                mapping.insert(new_path, *fb);
            },
            FallbackSpecNode::Pass(sub_fbs) => {
                listify_fallback_spec_helper(sub_fbs, new_path, mapping);
            },
            FallbackSpecNode::Both(fb, sub_fbs) => {
                mapping.insert(new_path.clone(), *fb);
                listify_fallback_spec_helper(sub_fbs, new_path, mapping);
            },
        }
    }
}

pub fn process_fallbacks<P: AsRef<Path>, S: AsRef<str>>(
    start_item_path: P,
    meta_format: MetaFormat,
    selection: &Selection,
    sort_order: SortOrder,
    fallback_spec: &FallbackSpec,
    default_fallback: Fallback,
    map_root_key: S,
) -> MetaBlock
{
    // Load the origin metadata.
    // This is the isolated metadata block for just the starting item.
    let origin_mb = MetaProcessor::process_item_file(
        start_item_path,
        meta_format,
        selection,
        sort_order,
        map_root_key,
    ).unwrap();

    MetaBlock::new()
}

#[cfg(test)]
mod tests {
    use super::Fallback;
    use super::FallbackSpec;
    use super::FallbackSpecNode;
    use super::InheritFallback;
    use super::HarvestFallback;

    #[test]
    fn test_listify_fallback_spec() {
        let title_key = String::from("title");
        let rg_key = String::from("rg");
        let peak_key = String::from("peak");

        let inputs_and_expected = vec![
            (
                hashmap![],
                hashmap![],
            ),
            (
                hashmap![
                    title_key.clone() => FallbackSpecNode::Leaf(Fallback::Inherit(InheritFallback::Override)),
                ],
                hashmap![
                    vec![&title_key] =>
                        Fallback::Inherit(InheritFallback::Override),
                ],
            ),
            (
                hashmap![
                    title_key.clone() => FallbackSpecNode::Leaf(Fallback::Inherit(InheritFallback::Override)),
                    rg_key.clone() => FallbackSpecNode::Both(
                        Fallback::Inherit(InheritFallback::Merge),
                        hashmap![
                            peak_key.clone() => FallbackSpecNode::Leaf(Fallback::Harvest(HarvestFallback::First)),
                        ],
                    ),
                ],
                hashmap![
                    vec![&title_key] =>
                        Fallback::Inherit(InheritFallback::Override),
                    vec![&rg_key] =>
                        Fallback::Inherit(InheritFallback::Merge),
                    vec![&rg_key, &peak_key] =>
                        Fallback::Harvest(HarvestFallback::First),
                ],
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = super::listify_fallback_spec(&input);
            assert_eq!(expected, produced);
        }
    }
}
