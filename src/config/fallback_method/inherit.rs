use std::collections::BTreeMap;

use metadata::types::MetaVal;
use metadata::types::MetaKey;

/// Different ways to process parent metadata into desired outputs.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InheritMethod {
    Overwrite,
    Merge,
}

impl InheritMethod {
    pub fn process<II>(self, mvs: II) -> MetaVal
    where
        II: IntoIterator<Item = MetaVal>,
    {
        // NOTE: Assume that the iterable is ordered from oldest to newest parent.
        mvs.into_iter().fold(MetaVal::Nil, |prev_mv, next_mv| {
            match self {
                InheritMethod::Overwrite => overwrite(prev_mv, next_mv),
                InheritMethod::Merge => merge(prev_mv, next_mv),
            }
        })
    }
}

/// Combines two meta values together following overwriting rules.
fn overwrite(_prev_mv: MetaVal, next_mv: MetaVal) -> MetaVal {
    next_mv
}

/// Combines two meta values together following merging rules.
fn merge(prev_mv: MetaVal, next_mv: MetaVal) -> MetaVal {
    match (prev_mv, next_mv) {
        (MetaVal::Map(prev_mv_map), MetaVal::Map(mut next_mv_map)) => {
            let mut merged = BTreeMap::new();

            for (prev_k, prev_v) in prev_mv_map.into_iter() {
                // Check if the key is contained in the new map.
                if let Some(next_v) = next_mv_map.remove(&prev_k) {
                    // Merge these two values together to get a result.
                    let merged_mv = merge(prev_v, next_v);
                    merged.insert(prev_k, merged_mv);
                }
                else {
                    // Just insert the old mapping value into the merged map.
                    merged.insert(prev_k, prev_v);
                }
            }

            // Drain all remaining entries from the new map and add them to the merged mapping.
            for (next_k, next_v) in next_mv_map {
                merged.insert(next_k, next_v);
            }

            MetaVal::Map(merged)
        },
        (MetaVal::Map(mut prev_mv_map), root_val_new) => {
            let merged_root_val = if let Some(root_val_old) = prev_mv_map.remove(&MetaKey::Nil) {
                merge(root_val_old, root_val_new)
            }
            else {
                root_val_new
            };

            prev_mv_map.insert(MetaKey::Nil, merged_root_val);

            MetaVal::Map(prev_mv_map)
        },
        (root_val_old, MetaVal::Map(mut next_mv_map)) => {
            let merged_root_val = if let Some(root_val_new) = next_mv_map.remove(&MetaKey::Nil) {
                merge(root_val_old, root_val_new)
            }
            else {
                root_val_old
            };

            next_mv_map.insert(MetaKey::Nil, merged_root_val);

            MetaVal::Map(next_mv_map)
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
            let (prev_mv, next_mv) = input;
            let produced = super::overwrite(prev_mv, next_mv);
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
            let (prev_mv, next_mv) = input;
            let produced = super::merge(prev_mv, next_mv);
            assert_eq!(expected, produced);
        }
    }
}
