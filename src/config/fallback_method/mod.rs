//! Representation and processing logic for metadata fallbacks.
//! A fallback is a way to obtain metadata from another source if it is missing for an item.

mod util;

use std::path::Path;
use std::collections::HashMap;

use config::meta_format::MetaFormat;
use config::selection::Selection;
use config::sort_order::SortOrder;
use metadata::types::MetaBlock;
use metadata::types::MetaKey;
use metadata::processor::MetaProcessor;

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

fn listify_fallback_spec(fallback_spec: &FallbackSpec) -> HashMap<Vec<&MetaKey>, Option<Fallback>> {
    let mut mapping = HashMap::new();

    listify_fallback_spec_helper(fallback_spec, vec![], &mut mapping);

    mapping
}

fn listify_fallback_spec_helper<'a>(
    fallback_spec: &'a HashMap<MetaKey, FallbackSpecNode>,
    curr_path: Vec<&'a MetaKey>,
    mapping: &mut HashMap<Vec<&'a MetaKey>, Option<Fallback>>)
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

#[cfg(test)]
mod tests {
    use super::Fallback;
    use super::FallbackSpecNode;
    use super::InheritFallback;
    use super::HarvestFallback;

    use metadata::types::MetaKey;

    #[test]
    fn test_listify_fallback_spec() {
        let title_key = MetaKey::from("title");
        let rg_key = MetaKey::from("rg");
        let peak_key = MetaKey::from("peak");

        let inputs_and_expected = vec![
            (
                hashmap![],
                hashmap![],
            ),
            (
                hashmap![
                    title_key.clone() => FallbackSpecNode::Leaf(Some(Fallback::Inherit(InheritFallback::Override))),
                ],
                hashmap![
                    vec![&title_key] =>
                        Some(Fallback::Inherit(InheritFallback::Override)),
                ],
            ),
            (
                hashmap![
                    title_key.clone() => FallbackSpecNode::Leaf(Some(Fallback::Inherit(InheritFallback::Override))),
                    rg_key.clone() => FallbackSpecNode::Both(
                        Some(Fallback::Inherit(InheritFallback::Merge)),
                        hashmap![
                            peak_key.clone() => FallbackSpecNode::Leaf(Some(Fallback::Harvest(HarvestFallback::First))),
                        ],
                    ),
                ],
                hashmap![
                    vec![&title_key] =>
                        Some(Fallback::Inherit(InheritFallback::Override)),
                    vec![&rg_key] =>
                        Some(Fallback::Inherit(InheritFallback::Merge)),
                    vec![&rg_key, &peak_key] =>
                        Some(Fallback::Harvest(HarvestFallback::First)),
                ],
            ),
            (
                hashmap![
                    title_key.clone() => FallbackSpecNode::Leaf(Some(Fallback::Inherit(InheritFallback::Override))),
                    rg_key.clone() => FallbackSpecNode::Pass(
                        hashmap![
                            peak_key.clone() => FallbackSpecNode::Leaf(Some(Fallback::Harvest(HarvestFallback::First))),
                        ],
                    ),
                ],
                hashmap![
                    vec![&title_key] =>
                        Some(Fallback::Inherit(InheritFallback::Override)),
                    vec![&rg_key, &peak_key] =>
                        Some(Fallback::Harvest(HarvestFallback::First)),
                ],
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = super::listify_fallback_spec(&input);
            assert_eq!(expected, produced);
        }
    }
}
