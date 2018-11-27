use std::collections::BTreeMap;
use std::collections::HashMap;
use std::path::Ancestors;
use std::path::Path;
use std::path::PathBuf;

use metadata::types::MetaVal;
use metadata::types::MetaKey;
use metadata::types::MetaBlock;
use config::meta_format::MetaFormat;
use config::selection::Selection;
use config::sort_order::SortOrder;
use metadata::processor::MetaProcessor;

/// Different ways to process parent metadata into desired outputs.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InheritMethod {
    Overwrite,
    Merge,
}

impl Default for InheritMethod {
    fn default() -> Self {
        InheritMethod::Overwrite
    }
}

impl InheritMethod {
    pub fn process<P: AsRef<Path>>(
        start_item_path: P,
        meta_format: MetaFormat,
        selection: &Selection,
        sort_order: SortOrder,
        method_map: &HashMap<String, InheritMethod>,
    ) -> MetaBlock
    {
        start_item_path
            .as_ref()
            .ancestors()
            .filter_map(|curr_item_path| {
                match MetaProcessor::process_item_file(
                    curr_item_path,
                    meta_format,
                    selection,
                    sort_order,
                ) {
                    Ok(mb) => Some(mb),
                    Err(err) => {
                        warn!("{}", err);
                        None
                    },
                }
            })
            .fold(MetaBlock::new(), |mut newer_mb, older_mb| {
                let mut combined_mb = MetaBlock::new();

                // Process each key-value pair in the older meta block.
                for (older_key, older_val) in older_mb {
                    // Check if the key is contained in the newer meta block.
                    if let Some(newer_val) = newer_mb.remove(&older_key) {
                        // Get the method used for this key.
                        // LEARN: Since the value in this mapping implements `Copy`, the `.cloned()` call should be cheap.
                        let i_method = method_map.get(&older_key).cloned().unwrap_or_default();

                        let combined_val = match i_method {
                            InheritMethod::Overwrite => overwrite(older_val, newer_val),
                            InheritMethod::Merge => merge(older_val, newer_val),
                        };

                        combined_mb.insert(older_key, combined_val);
                    }
                    else {
                        // Just move the older value over.
                        combined_mb.insert(older_key, older_val);
                    }
                }

                // Drain any remaining key-value pairs from the newer meta block.
                for (newer_key, newer_val) in newer_mb {
                    combined_mb.insert(newer_key, newer_val);
                }

                combined_mb
            })
    }
}

/// Combines two meta values together following overwriting rules.
fn overwrite(_older_mv: MetaVal, newer_mv: MetaVal) -> MetaVal {
    newer_mv
}

/// Combines two meta values together following merging rules.
fn merge(older_mv: MetaVal, newer_mv: MetaVal) -> MetaVal {
    match (older_mv, newer_mv) {
        (MetaVal::Map(older_mv_map), MetaVal::Map(mut newer_mv_map)) => {
            let mut merged = BTreeMap::new();

            for (older_k, older_v) in older_mv_map.into_iter() {
                // Check if the key is contained in the new map.
                if let Some(newer_v) = newer_mv_map.remove(&older_k) {
                    // Merge these two values together to get a result.
                    let merged_mv = merge(older_v, newer_v);
                    merged.insert(older_k, merged_mv);
                }
                else {
                    // Just insert the old mapping value into the merged map.
                    merged.insert(older_k, older_v);
                }
            }

            // Drain all remaining entries from the new map and add them to the merged mapping.
            for (newer_k, newer_v) in newer_mv_map {
                merged.insert(newer_k, newer_v);
            }

            MetaVal::Map(merged)
        },
        (MetaVal::Map(mut older_mv_map), root_val_new) => {
            let merged_root_val = if let Some(root_val_old) = older_mv_map.remove(&MetaKey::Nil) {
                merge(root_val_old, root_val_new)
            }
            else {
                root_val_new
            };

            older_mv_map.insert(MetaKey::Nil, merged_root_val);

            MetaVal::Map(older_mv_map)
        },
        (root_val_old, MetaVal::Map(mut newer_mv_map)) => {
            let merged_root_val = if let Some(root_val_new) = newer_mv_map.remove(&MetaKey::Nil) {
                merge(root_val_old, root_val_new)
            }
            else {
                root_val_old
            };

            newer_mv_map.insert(MetaKey::Nil, merged_root_val);

            MetaVal::Map(newer_mv_map)
        },
        (_o, n) => {
            n
        },
    }
}

#[cfg(test)]
mod tests {
    use metadata::types::MetaVal;
    use metadata::types::MetaKey;

    #[test]
    fn test_override() {
        let inputs_and_expected = vec![
            (
                (
                    MetaVal::Nil,
                    MetaVal::Str(String::from("A")),
                ),
                MetaVal::Str(String::from("A")),
            ),
            (
                (
                    MetaVal::Str(String::from("A")),
                    MetaVal::Map(btreemap![
                        MetaKey::Str(String::from("b")) => MetaVal::Str(String::from("B")),
                    ]),
                ),
                MetaVal::Map(btreemap![
                    MetaKey::Str(String::from("b")) => MetaVal::Str(String::from("B")),
                ]),
            ),
            (
                (
                    MetaVal::Map(btreemap![
                        MetaKey::Str(String::from("a")) => MetaVal::Str(String::from("A")),
                    ]),
                    MetaVal::Map(btreemap![
                        MetaKey::Str(String::from("b")) => MetaVal::Str(String::from("B")),
                    ]),
                ),
                MetaVal::Map(btreemap![
                    // MetaKey::Nil => MetaVal::Str(String::from("A")),
                    MetaKey::Str(String::from("b")) => MetaVal::Str(String::from("B")),
                ]),
            ),
            (
                (
                    MetaVal::Map(btreemap![
                        MetaKey::Nil => MetaVal::Map(btreemap![
                            MetaKey::Nil => MetaVal::Str(String::from("X")),
                            MetaKey::Str(String::from("y")) => MetaVal::Str(String::from("Y")),
                        ]),
                        MetaKey::Str(String::from("a")) => MetaVal::Str(String::from("A")),
                    ]),
                    MetaVal::Map(btreemap![
                        MetaKey::Nil => MetaVal::Map(btreemap![
                            MetaKey::Str(String::from("z")) => MetaVal::Str(String::from("Z")),
                        ]),
                        MetaKey::Str(String::from("b")) => MetaVal::Str(String::from("B")),
                    ]),
                ),
                MetaVal::Map(btreemap![
                    MetaKey::Nil => MetaVal::Map(btreemap![
                        MetaKey::Str(String::from("z")) => MetaVal::Str(String::from("Z")),
                    ]),
                    MetaKey::Str(String::from("b")) => MetaVal::Str(String::from("B")),
                ]),
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let (older_mv, newer_mv) = input;
            let produced = super::overwrite(older_mv, newer_mv);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_merge() {
        let inputs_and_expected = vec![
            (
                (
                    MetaVal::Nil,
                    MetaVal::Str(String::from("A")),
                ),
                MetaVal::Str(String::from("A")),
            ),
            (
                (
                    MetaVal::Str(String::from("A")),
                    MetaVal::Map(btreemap![
                        MetaKey::Str(String::from("b")) => MetaVal::Str(String::from("B")),
                    ]),
                ),
                MetaVal::Map(btreemap![
                    MetaKey::Nil => MetaVal::Str(String::from("A")),
                    MetaKey::Str(String::from("b")) => MetaVal::Str(String::from("B")),
                ]),
            ),
            (
                (
                    MetaVal::Map(btreemap![
                        MetaKey::Str(String::from("a")) => MetaVal::Str(String::from("A")),
                    ]),
                    MetaVal::Map(btreemap![
                        MetaKey::Str(String::from("b")) => MetaVal::Str(String::from("B")),
                    ]),
                ),
                MetaVal::Map(btreemap![
                    MetaKey::Str(String::from("a")) => MetaVal::Str(String::from("A")),
                    MetaKey::Str(String::from("b")) => MetaVal::Str(String::from("B")),
                ]),
            ),
            (
                (
                    MetaVal::Map(btreemap![
                        MetaKey::Nil => MetaVal::Map(btreemap![
                            MetaKey::Nil => MetaVal::Str(String::from("X")),
                            MetaKey::Str(String::from("y")) => MetaVal::Str(String::from("Y")),
                        ]),
                        MetaKey::Str(String::from("a")) => MetaVal::Str(String::from("A")),
                    ]),
                    MetaVal::Map(btreemap![
                        MetaKey::Nil => MetaVal::Map(btreemap![
                            MetaKey::Str(String::from("z")) => MetaVal::Str(String::from("Z")),
                        ]),
                        MetaKey::Str(String::from("b")) => MetaVal::Str(String::from("B")),
                    ]),
                ),
                MetaVal::Map(btreemap![
                    MetaKey::Nil => MetaVal::Map(btreemap![
                        MetaKey::Nil => MetaVal::Str(String::from("X")),
                        MetaKey::Str(String::from("y")) => MetaVal::Str(String::from("Y")),
                        MetaKey::Str(String::from("z")) => MetaVal::Str(String::from("Z")),
                    ]),
                    MetaKey::Str(String::from("a")) => MetaVal::Str(String::from("A")),
                    MetaKey::Str(String::from("b")) => MetaVal::Str(String::from("B")),
                ]),
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let (older_mv, newer_mv) = input;
            let produced = super::merge(older_mv, newer_mv);
            assert_eq!(expected, produced);
        }
    }
}
