//! Types for modeling and representing item metadata.

use std::collections::BTreeMap;
use std::collections::HashMap;

use util::GenConverter;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone)]
pub enum MetaKey {
    Nil,
    Str(String),
}

impl MetaKey {
    pub fn iter_over<'a>(&'a self) -> impl Iterator<Item = &'a String> {
        let closure = move || {
            match *self {
                MetaKey::Nil => {},
                MetaKey::Str(ref s) => { yield s; },
            }
        };

        GenConverter::gen_to_iter(closure)
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum MetaValue {
    Nil,
    Str(String),
    Seq(Vec<MetaValue>),
    Map(BTreeMap<MetaKey, MetaValue>),
}

impl MetaValue {
    pub fn iter_over<'a>(&'a self, mis: MappingIterScheme) -> impl Iterator<Item = &'a String> {
        // LEARN: The `Box::new()` calls are to allow the generator to be recursive.
        let closure = move || {
            match *self {
                MetaValue::Nil => {},
                MetaValue::Str(ref s) => { yield s; },
                MetaValue::Seq(ref mvs) => {
                    for mv in mvs {
                        for i in Box::new(mv.iter_over(mis)) {
                            yield i;
                        }
                    }
                },
                MetaValue::Map(ref map) => {
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

pub type MetaBlock = BTreeMap<String, MetaValue>;
pub type MetaBlockSeq = Vec<MetaBlock>;
pub type MetaBlockMap = HashMap<String, MetaBlock>;

/// A data structure-level representation of all possible metadata types and their formats.
/// This is intended to be independent of the text-level representation of the metadata.
#[derive(Debug)]
pub enum MetaStructure {
    One(MetaBlock),
    Seq(MetaBlockSeq),
    Map(MetaBlockMap),
}
