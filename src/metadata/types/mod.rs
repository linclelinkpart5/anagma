//! Types for modeling and representing item metadata.

use std::collections::BTreeMap;
use std::collections::HashMap;

use util::GenConverter;

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

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum MetaVal {
    Nil,
    Str(String),
    Seq(Vec<MetaVal>),
    Map(BTreeMap<String, MetaVal>),
}

impl MetaVal {
    /// Combines two meta values together following inheriting rules.
    pub fn inherit(self, opt_new_mv: Option<MetaVal>) -> MetaVal {
        opt_new_mv.unwrap_or(self)
    }

    /// Combines two meta values together following merging rules.
    pub fn merge<S: AsRef<str>>(self, opt_new_mv: Option<MetaVal>, map_root_key: S) -> MetaVal {
        let map_root_key = map_root_key.as_ref();
        let old_mv = self;

        if let Some(new_mv) = opt_new_mv {
            match (old_mv, new_mv) {
                (MetaVal::Map(mut mv_old_map), MetaVal::Map(mut mv_new_map)) => {
                    let mut merged = BTreeMap::new();

                    for (k_old, v_old) in mv_old_map.into_iter() {
                        // Check if the key is contained in the new map.
                        if let Some(v_new) = mv_new_map.remove(&k_old) {
                            // Merge these two values together to get a result.
                            let merged_mv = v_old.merge(Some(v_new), &map_root_key);
                            merged.insert(k_old, merged_mv);
                        }
                        else {
                            // Just insert the old mapping value into the merged map.
                            merged.insert(k_old, v_old);
                        }
                    }

                    // Drain all remaining entries from the new map and add them to the merged mapping.
                    for (k_new, v_new) in mv_new_map {
                        merged.insert(k_new, v_new);
                    }

                    MetaVal::Map(merged)
                },
                (MetaVal::Map(mut mv_old_map), root_val_new) => {
                    let merged_root_val = if let Some(root_val_old) = mv_old_map.remove(map_root_key) {
                        root_val_old.merge(Some(root_val_new), &map_root_key)
                    }
                    else {
                        root_val_new
                    };

                    mv_old_map.insert(map_root_key.to_string(), merged_root_val);

                    MetaVal::Map(mv_old_map)
                },
                (root_val_old, MetaVal::Map(mut mv_new_map)) => {
                    let merged_root_val = if let Some(root_val_new) = mv_new_map.remove(map_root_key) {
                        root_val_old.merge(Some(root_val_new), &map_root_key)
                    }
                    else {
                        root_val_old
                    };

                    mv_new_map.insert(map_root_key.to_string(), merged_root_val);

                    MetaVal::Map(mv_new_map)
                },
                (_o, n) => {
                    n
                },
            }
        }
        else {
            old_mv
        }
    }

    pub fn iter_over<'a>(&'a self, mis: MappingIterScheme) -> impl Iterator<Item = &'a String> {
        // LEARN: The `Box::new()` calls are to allow the generator to be recursive.
        let closure = move || {
            match *self {
                MetaVal::Nil => {},
                MetaVal::Str(ref s) => { yield s; },
                MetaVal::Seq(ref mvs) => {
                    for mv in mvs {
                        for i in Box::new(mv.iter_over(mis)) {
                            yield i;
                        }
                    }
                },
                MetaVal::Map(ref map) => {
                    for (mk, mv) in map {
                        match mis {
                            MappingIterScheme::Keys | MappingIterScheme::Both => {
                                // This outputs the value of the Nil key first, but only if a BTreeMap is used.
                                yield mk;
                            },
                            MappingIterScheme::Vals => {},
                        };

                        match mis {
                            MappingIterScheme::Vals | MappingIterScheme::Both => {
                                for s in Box::new(mv.iter_over(mis)) {
                                    yield s;
                                }
                            },
                            MappingIterScheme::Keys => {},
                        };
                    }
                },
            }
        };

        GenConverter::gen_to_iter(closure)
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub enum MappingIterScheme {
    Keys,
    Vals,
    Both,
}

#[cfg(test)]
mod tests {
    use super::MetaVal;
    use super::MappingIterScheme;

    #[test]
    fn test_inherit() {
        let inputs_and_expected = vec![
            (
                (
                    MetaVal::Nil,
                    Some(MetaVal::Str(String::from("A"))),
                ),
                MetaVal::Str(String::from("A")),
            ),
            (
                (
                    MetaVal::Nil,
                    None,
                ),
                MetaVal::Nil,
            ),
            (
                (
                    MetaVal::Str(String::from("A")),
                    Some(MetaVal::Map(btreemap![
                        String::from("b") => MetaVal::Str(String::from("B")),
                    ])),
                ),
                MetaVal::Map(btreemap![
                    String::from("b") => MetaVal::Str(String::from("B")),
                ]),
            ),
            (
                (
                    MetaVal::Map(btreemap![
                        String::from("a") => MetaVal::Str(String::from("A")),
                    ]),
                    Some(MetaVal::Map(btreemap![
                        String::from("b") => MetaVal::Str(String::from("B")),
                    ])),
                ),
                MetaVal::Map(btreemap![
                    // String::from("~") => MetaVal::Str(String::from("A")),
                    String::from("b") => MetaVal::Str(String::from("B")),
                ]),
            ),
            (
                (
                    MetaVal::Map(btreemap![
                        String::from("~") => MetaVal::Map(btreemap![
                            String::from("~") => MetaVal::Str(String::from("X")),
                            String::from("y") => MetaVal::Str(String::from("Y")),
                        ]),
                        String::from("a") => MetaVal::Str(String::from("A")),
                    ]),
                    Some(MetaVal::Map(btreemap![
                        String::from("~") => MetaVal::Map(btreemap![
                            String::from("z") => MetaVal::Str(String::from("Z")),
                        ]),
                        String::from("b") => MetaVal::Str(String::from("B")),
                    ])),
                ),
                MetaVal::Map(btreemap![
                    String::from("~") => MetaVal::Map(btreemap![
                        String::from("z") => MetaVal::Str(String::from("Z")),
                    ]),
                    String::from("b") => MetaVal::Str(String::from("B")),
                ]),
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let (old_mv, new_mv) = input;
            let produced = old_mv.inherit(new_mv);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_merge() {
        let inputs_and_expected = vec![
            (
                (
                    MetaVal::Nil,
                    Some(MetaVal::Str(String::from("A"))),
                ),
                MetaVal::Str(String::from("A")),
            ),
            (
                (
                    MetaVal::Nil,
                    None,
                ),
                MetaVal::Nil,
            ),
            (
                (
                    MetaVal::Str(String::from("A")),
                    Some(MetaVal::Map(btreemap![
                        String::from("b") => MetaVal::Str(String::from("B")),
                    ])),
                ),
                MetaVal::Map(btreemap![
                    String::from("~") => MetaVal::Str(String::from("A")),
                    String::from("b") => MetaVal::Str(String::from("B")),
                ]),
            ),
            (
                (
                    MetaVal::Map(btreemap![
                        String::from("a") => MetaVal::Str(String::from("A")),
                    ]),
                    Some(MetaVal::Map(btreemap![
                        String::from("b") => MetaVal::Str(String::from("B")),
                    ])),
                ),
                MetaVal::Map(btreemap![
                    String::from("a") => MetaVal::Str(String::from("A")),
                    String::from("b") => MetaVal::Str(String::from("B")),
                ]),
            ),
            (
                (
                    MetaVal::Map(btreemap![
                        String::from("~") => MetaVal::Map(btreemap![
                            String::from("~") => MetaVal::Str(String::from("X")),
                            String::from("y") => MetaVal::Str(String::from("Y")),
                        ]),
                        String::from("a") => MetaVal::Str(String::from("A")),
                    ]),
                    Some(MetaVal::Map(btreemap![
                        String::from("~") => MetaVal::Map(btreemap![
                            String::from("z") => MetaVal::Str(String::from("Z")),
                        ]),
                        String::from("b") => MetaVal::Str(String::from("B")),
                    ])),
                ),
                MetaVal::Map(btreemap![
                    String::from("~") => MetaVal::Map(btreemap![
                        String::from("~") => MetaVal::Str(String::from("X")),
                        String::from("y") => MetaVal::Str(String::from("Y")),
                        String::from("z") => MetaVal::Str(String::from("Z")),
                    ]),
                    String::from("a") => MetaVal::Str(String::from("A")),
                    String::from("b") => MetaVal::Str(String::from("B")),
                ]),
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let (old_mv, new_mv) = input;
            let produced = old_mv.merge(new_mv, "~");
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_iter_over() {
        let inputs_and_expected = vec![
            (
                MetaVal::Nil,
                vec![],
            ),
            (
                MetaVal::Str("val".to_string()),
                vec!["val"],
            ),
            (
                MetaVal::Seq(vec![
                    MetaVal::Str("val_a".to_string()),
                    MetaVal::Str("val_b".to_string()),
                    MetaVal::Str("val_c".to_string()),
                ]),
                vec!["val_a", "val_b", "val_c"],
            ),
            (
                MetaVal::Seq(vec![
                    MetaVal::Str("val_a".to_string()),
                    MetaVal::Seq(vec![
                        MetaVal::Str("val_b".to_string()),
                        MetaVal::Str("val_c".to_string()),
                    ]),
                    MetaVal::Str("val_d".to_string()),
                ]),
                vec!["val_a", "val_b", "val_c", "val_d"],
            ),
            (
                MetaVal::Seq(vec![
                    MetaVal::Str("val_a".to_string()),
                    MetaVal::Nil,
                    MetaVal::Seq(vec![
                        MetaVal::Str("val_b".to_string()),
                        MetaVal::Str("val_c".to_string()),
                        MetaVal::Nil,
                    ]),
                    MetaVal::Str("val_d".to_string()),
                    MetaVal::Nil,
                ]),
                vec!["val_a", "val_b", "val_c", "val_d"],
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let produced: Vec<_> = input.iter_over(MappingIterScheme::Both).collect();
            assert_eq!(expected, produced);
        }

        // For testing mappings.
        let map_a = MetaVal::Map(btreemap![
            String::from("~") => MetaVal::Str("val_x".to_string()),
            "key_d".to_string() => MetaVal::Nil,
            "key_c".to_string() => MetaVal::Str("val_c".to_string()),
            "key_b".to_string() => MetaVal::Str("val_b".to_string()),
            "key_a".to_string() => MetaVal::Str("val_a".to_string()),
        ]);

        let map_b = MetaVal::Map(btreemap![
            String::from("~") => MetaVal::Seq(vec![
                MetaVal::Str("val_a".to_string()),
                MetaVal::Str("val_b".to_string()),
                MetaVal::Str("val_c".to_string()),
                MetaVal::Str("val_d".to_string()),
            ]),
            "key_d".to_string() => MetaVal::Str("val_k".to_string()),
            "key_c".to_string() => MetaVal::Seq(vec![
                MetaVal::Str("val_j".to_string()),
            ]),
            "key_b".to_string() => MetaVal::Seq(vec![
                MetaVal::Str("val_h".to_string()),
                MetaVal::Str("val_i".to_string()),
            ]),
            "key_a".to_string() => MetaVal::Seq(vec![
                MetaVal::Str("val_e".to_string()),
                MetaVal::Str("val_f".to_string()),
                MetaVal::Str("val_g".to_string()),
            ]),
        ]);

        let map_c = MetaVal::Map(btreemap![
            "key_d".to_string() => MetaVal::Map(btreemap![
                String::from("~") => MetaVal::Str("val_e".to_string()),
                "key_e".to_string() => MetaVal::Str("val_f".to_string()),
                "key_f".to_string() => MetaVal::Str("val_g".to_string()),
                "key_g".to_string() => MetaVal::Str("val_h".to_string()),
            ]),
            String::from("~") => MetaVal::Map(btreemap![
                String::from("~") => MetaVal::Str("val_a".to_string()),
                "key_a".to_string() => MetaVal::Str("val_b".to_string()),
                "key_b".to_string() => MetaVal::Str("val_c".to_string()),
                "key_c".to_string() => MetaVal::Str("val_d".to_string()),
            ]),
        ]);

        let inputs_and_expected = vec![
            (
                (map_a.clone(), MappingIterScheme::Both),
                vec!["key_a", "val_a", "key_b", "val_b", "key_c", "val_c", "key_d", "~", "val_x"],
            ),
            (
                (map_a.clone(), MappingIterScheme::Keys),
                vec!["key_a", "key_b", "key_c", "key_d", "~"],
            ),
            (
                (map_a.clone(), MappingIterScheme::Vals),
                vec!["val_a", "val_b", "val_c", "val_x"],
            ),
            (
                (map_b.clone(), MappingIterScheme::Both),
                vec![
                    "key_a", "val_e", "val_f", "val_g",
                    "key_b", "val_h", "val_i", "key_c", "val_j", "key_d", "val_k",
                    "~", "val_a", "val_b", "val_c", "val_d"
                ],
            ),
            (
                (map_b.clone(), MappingIterScheme::Keys),
                vec!["key_a", "key_b", "key_c", "key_d", "~"],
            ),
            (
                (map_b.clone(), MappingIterScheme::Vals),
                vec![
                    "val_e", "val_f",
                    "val_g", "val_h", "val_i", "val_j", "val_k",
                    "val_a", "val_b", "val_c", "val_d",
                ],
            ),
            (
                (map_c.clone(), MappingIterScheme::Both),
                vec![
                    "key_d", "key_e", "val_f", "key_f", "val_g",
                    "key_g", "val_h", "~", "val_e", "~", "key_a",
                    "val_b", "key_b", "val_c", "key_c", "val_d", "~", "val_a",
                ],
            ),
            (
                (map_c.clone(), MappingIterScheme::Keys),
                vec![
                    "key_d", "~",
                    // TODO: Should this be the following (left-side-hugging) instead?
                    // "key_a", "key_b", "key_c", "key_d", "key_e", "key_f", "key_g",
                ],
            ),
            (
                (map_c.clone(), MappingIterScheme::Vals),
                vec![
                    "val_f", "val_g", "val_h", "val_e", "val_b", "val_c", "val_d", "val_a",
                ],
            ),
        ];

        for ((input, mis), expected) in inputs_and_expected {
            let produced: Vec<_> = input.iter_over(mis).collect();
            assert_eq!(expected, produced);
        }
    }
}
