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
                                for s in Box::new(mk.iter_over()) {
                                    yield s;
                                }
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
    fn test_iter_over() {
        let inputs_and_expected = vec![
            // Nil expands to no string values.
            (
                MetaVal::Nil,
                vec![],
            ),
            // Str expands into exactly one string value.
            (
                MetaVal::Str("sample".to_string()),
                vec!["sample"],
            ),
            (
                MetaVal::Seq(vec![
                    MetaVal::Str("sample_a".to_string()),
                    MetaVal::Str("sample_b".to_string()),
                    MetaVal::Str("sample_c".to_string()),
                ]),
                vec!["sample_a", "sample_b", "sample_c"],
            ),
            (
                MetaVal::Seq(vec![
                    MetaVal::Str("sample_a".to_string()),
                    MetaVal::Seq(vec![
                        MetaVal::Str("sample_b".to_string()),
                        MetaVal::Str("sample_c".to_string()),
                    ]),
                    MetaVal::Str("sample_d".to_string()),
                ]),
                vec!["sample_a", "sample_b", "sample_c", "sample_d"],
            ),
            (
                MetaVal::Seq(vec![
                    MetaVal::Str("sample_a".to_string()),
                    MetaVal::Nil,
                    MetaVal::Seq(vec![
                        MetaVal::Str("sample_b".to_string()),
                        MetaVal::Str("sample_c".to_string()),
                        MetaVal::Nil,
                    ]),
                    MetaVal::Str("sample_d".to_string()),
                    MetaVal::Nil,
                ]),
                vec!["sample_a", "sample_b", "sample_c", "sample_d"],
            ),
        ];

        // Not relevant for this batch of test cases.
        let mis = MappingIterScheme::Both;
        for (input, expected) in inputs_and_expected {
            let produced: Vec<_> = input.iter_over(mis).collect();
            assert_eq!(expected, produced);
        }
    }
}
