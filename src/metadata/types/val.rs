//! Types for modeling and representing item metadata.

use std::collections::BTreeMap;

use util::GenConverter;
use metadata::types::key::MetaKey;

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum MetaVal {
    Nil,
    Str(String),
    Seq(Vec<MetaVal>),
    Map(BTreeMap<MetaKey, MetaVal>),
}

impl MetaVal {
    // pub fn iter_over<'a>(&'a self, mis: MappingIterScheme) -> impl Iterator<Item = &'a String> {
    //     // LEARN: The `Box::new()` calls are to allow the generator to be recursive.
    //     let closure = move || {
    //         match *self {
    //             MetaVal::Nil => {},
    //             MetaVal::Str(ref s) => { yield s; },
    //             MetaVal::Seq(ref mvs) => {
    //                 for mv in mvs {
    //                     for i in Box::new(mv.iter_over(mis)) {
    //                         yield i;
    //                     }
    //                 }
    //             },
    //             MetaVal::Map(ref map) => {
    //                 for (mk, mv) in map {
    //                     match mis {
    //                         MappingIterScheme::Keys | MappingIterScheme::Both => {
    //                             // This outputs the value of the Nil key first, but only if a BTreeMap is used.
    //                             for s in Box::new(mk.iter_over()) {
    //                                 yield s;
    //                             }
    //                         },
    //                         MappingIterScheme::Vals => {},
    //                     };

    //                     match mis {
    //                         MappingIterScheme::Vals | MappingIterScheme::Both => {
    //                             for s in Box::new(mv.iter_over(mis)) {
    //                                 yield s;
    //                             }
    //                         },
    //                         MappingIterScheme::Keys => {},
    //                     };
    //                 }
    //             },
    //         }
    //     };

    //     GenConverter::gen_to_iter(closure)
    // }
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

    use metadata::types::key::MetaKey;

    // #[test]
    // fn test_iter_over() {
    //     let inputs_and_expected = vec![
    //         (
    //             MetaVal::Nil,
    //             vec![],
    //         ),
    //         (
    //             MetaVal::Str("val".to_string()),
    //             vec!["val"],
    //         ),
    //         (
    //             MetaVal::Seq(vec![
    //                 MetaVal::Str("val_a".to_string()),
    //                 MetaVal::Str("val_b".to_string()),
    //                 MetaVal::Str("val_c".to_string()),
    //             ]),
    //             vec!["val_a", "val_b", "val_c"],
    //         ),
    //         (
    //             MetaVal::Seq(vec![
    //                 MetaVal::Str("val_a".to_string()),
    //                 MetaVal::Seq(vec![
    //                     MetaVal::Str("val_b".to_string()),
    //                     MetaVal::Str("val_c".to_string()),
    //                 ]),
    //                 MetaVal::Str("val_d".to_string()),
    //             ]),
    //             vec!["val_a", "val_b", "val_c", "val_d"],
    //         ),
    //         (
    //             MetaVal::Seq(vec![
    //                 MetaVal::Str("val_a".to_string()),
    //                 MetaVal::Nil,
    //                 MetaVal::Seq(vec![
    //                     MetaVal::Str("val_b".to_string()),
    //                     MetaVal::Str("val_c".to_string()),
    //                     MetaVal::Nil,
    //                 ]),
    //                 MetaVal::Str("val_d".to_string()),
    //                 MetaVal::Nil,
    //             ]),
    //             vec!["val_a", "val_b", "val_c", "val_d"],
    //         ),
    //     ];

    //     for (input, expected) in inputs_and_expected {
    //         let produced: Vec<_> = input.iter_over(MappingIterScheme::Both).collect();
    //         assert_eq!(expected, produced);
    //     }

    //     // For testing mappings.
    //     let map_a = MetaVal::Map(btreemap![
    //         MetaKey::Nil => MetaVal::Str("val_x".to_string()),
    //         MetaKey::Str("key_d".to_string()) => MetaVal::Nil,
    //         MetaKey::Str("key_c".to_string()) => MetaVal::Str("val_c".to_string()),
    //         MetaKey::Str("key_b".to_string()) => MetaVal::Str("val_b".to_string()),
    //         MetaKey::Str("key_a".to_string()) => MetaVal::Str("val_a".to_string()),
    //     ]);

    //     let map_b = MetaVal::Map(btreemap![
    //         MetaKey::Nil => MetaVal::Seq(vec![
    //             MetaVal::Str("val_a".to_string()),
    //             MetaVal::Str("val_b".to_string()),
    //             MetaVal::Str("val_c".to_string()),
    //             MetaVal::Str("val_d".to_string()),
    //         ]),
    //         MetaKey::Str("key_d".to_string()) => MetaVal::Str("val_k".to_string()),
    //         MetaKey::Str("key_c".to_string()) => MetaVal::Seq(vec![
    //             MetaVal::Str("val_j".to_string()),
    //         ]),
    //         MetaKey::Str("key_b".to_string()) => MetaVal::Seq(vec![
    //             MetaVal::Str("val_h".to_string()),
    //             MetaVal::Str("val_i".to_string()),
    //         ]),
    //         MetaKey::Str("key_a".to_string()) => MetaVal::Seq(vec![
    //             MetaVal::Str("val_e".to_string()),
    //             MetaVal::Str("val_f".to_string()),
    //             MetaVal::Str("val_g".to_string()),
    //         ]),
    //     ]);

    //     let map_c = MetaVal::Map(btreemap![
    //         MetaKey::Str("key_d".to_string()) => MetaVal::Map(btreemap![
    //             MetaKey::Nil => MetaVal::Str("val_e".to_string()),
    //             MetaKey::Str("key_e".to_string()) => MetaVal::Str("val_f".to_string()),
    //             MetaKey::Str("key_f".to_string()) => MetaVal::Str("val_g".to_string()),
    //             MetaKey::Str("key_g".to_string()) => MetaVal::Str("val_h".to_string()),
    //         ]),
    //         MetaKey::Nil => MetaVal::Map(btreemap![
    //             MetaKey::Nil => MetaVal::Str("val_a".to_string()),
    //             MetaKey::Str("key_a".to_string()) => MetaVal::Str("val_b".to_string()),
    //             MetaKey::Str("key_b".to_string()) => MetaVal::Str("val_c".to_string()),
    //             MetaKey::Str("key_c".to_string()) => MetaVal::Str("val_d".to_string()),
    //         ]),
    //     ]);

    //     let inputs_and_expected = vec![
    //         (
    //             (map_a.clone(), MappingIterScheme::Both),
    //             vec!["val_x", "key_a", "val_a", "key_b", "val_b", "key_c", "val_c", "key_d"],
    //         ),
    //         (
    //             (map_a.clone(), MappingIterScheme::Keys),
    //             vec!["key_a", "key_b", "key_c", "key_d"],
    //         ),
    //         (
    //             (map_a.clone(), MappingIterScheme::Vals),
    //             vec!["val_x", "val_a", "val_b", "val_c"],
    //         ),
    //         (
    //             (map_b.clone(), MappingIterScheme::Both),
    //             vec![
    //                 "val_a", "val_b", "val_c", "val_d", "key_a", "val_e", "val_f", "val_g",
    //                 "key_b", "val_h", "val_i", "key_c", "val_j", "key_d", "val_k",
    //             ],
    //         ),
    //         (
    //             (map_b.clone(), MappingIterScheme::Keys),
    //             vec!["key_a", "key_b", "key_c","key_d"],
    //         ),
    //         (
    //             (map_b.clone(), MappingIterScheme::Vals),
    //             vec![
    //                 "val_a", "val_b", "val_c", "val_d", "val_e", "val_f",
    //                 "val_g", "val_h", "val_i", "val_j", "val_k",
    //             ],
    //         ),
    //         (
    //             (map_c.clone(), MappingIterScheme::Both),
    //             vec![
    //                 "val_a", "key_a", "val_b", "key_b", "val_c", "key_c", "val_d", "key_d",
    //                 "val_e", "key_e", "val_f", "key_f", "val_g", "key_g", "val_h",
    //             ],
    //         ),
    //         (
    //             (map_c.clone(), MappingIterScheme::Keys),
    //             vec![
    //                 "key_d",
    //                 // TODO: Should this be the following (left-side-hugging) instead?
    //                 // "key_a", "key_b", "key_c", "key_d", "key_e", "key_f", "key_g",
    //             ],
    //         ),
    //         (
    //             (map_c.clone(), MappingIterScheme::Vals),
    //             vec![
    //                 "val_a", "val_b", "val_c", "val_d", "val_e", "val_f", "val_g", "val_h",
    //             ],
    //         ),
    //     ];

    //     for ((input, mis), expected) in inputs_and_expected {
    //         let produced: Vec<_> = input.iter_over(mis).collect();
    //         assert_eq!(expected, produced);
    //     }
    // }
}
