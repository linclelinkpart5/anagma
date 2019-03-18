//! These are helper structs and types used for deserializing metadata.
//! After deserialization, extra post-processing is done to convert these to their final used forms.

use std::collections::BTreeMap;
use std::collections::HashMap;

use bigdecimal::BigDecimal;

use metadata::types::MetaVal as RealMetaVal;
use metadata::types::MetaKey as RealMetaKey;
use metadata::types::MetaStructure as RealMetaStructure;
use metadata::types::MetaBlock as RealMetaBlock;

#[derive(PartialEq, Eq, Debug, Clone, Hash, Deserialize)]
#[serde(untagged)]
pub enum MetaVal {
    Nil,
    Str(String),
    Seq(Vec<MetaVal>),
    Map(BTreeMap<String, MetaVal>),
    Int(i64),
    Bul(bool),
    Dec(BigDecimal),
}

impl MetaVal {
    pub fn into_real_meta_val<S: AsRef<str>>(self, map_root_key: S) -> RealMetaVal {
        match self {
            MetaVal::Nil => RealMetaVal::Nil,
            MetaVal::Str(s) => RealMetaVal::Str(s),
            MetaVal::Seq(seq) => {
                RealMetaVal::Seq(
                    seq
                        .into_iter()
                        .map(|mv| mv.into_real_meta_val(map_root_key.as_ref()))
                        .collect()
                )
            },
            MetaVal::Map(map) => {
                // All occurences of the map root key must be converted into a null meta key.
                RealMetaVal::Map(
                    map
                        .into_iter()
                        .map(|(k, v)| {
                            (
                                match k == map_root_key.as_ref() {
                                    true => RealMetaKey::Nil,
                                    false => RealMetaKey::Str(k),
                                },
                                v.into_real_meta_val(map_root_key.as_ref())
                            )
                        })
                        .collect()
                )
            },
            MetaVal::Int(i) => RealMetaVal::Int(i),
            MetaVal::Bul(b) => RealMetaVal::Bul(b),
            MetaVal::Dec(d) => RealMetaVal::Dec(d),
        }
    }
}

pub type MetaBlock = BTreeMap<String, MetaVal>;
pub type MetaBlockSeq = Vec<MetaBlock>;
pub type MetaBlockMap = HashMap<String, MetaBlock>;

fn into_real_meta_block<S: AsRef<str>>(mb: MetaBlock, map_root_key: S) -> RealMetaBlock {
    mb
        .into_iter()
        .map(|(k, v)| {
            (
                match k == map_root_key.as_ref() {
                    true => RealMetaKey::Nil,
                    false => RealMetaKey::Str(k),
                },
                v.into_real_meta_val(map_root_key.as_ref()),
            )
        })
        .collect()
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub(crate) enum UnitMetaStructure {
    One(MetaBlock),
}

impl UnitMetaStructure {
    pub fn into_real_meta_structure<S: AsRef<str>>(self, map_root_key: S) -> RealMetaStructure {
        match self {
            UnitMetaStructure::One(mb) => RealMetaStructure::One(into_real_meta_block(mb, map_root_key)),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub(crate) enum ManyMetaStructure {
    Seq(MetaBlockSeq),
    Map(MetaBlockMap),
}

impl ManyMetaStructure {
    pub fn into_real_meta_structure<S: AsRef<str>>(self, map_root_key: S) -> RealMetaStructure {
        match self {
            ManyMetaStructure::Seq(mb_seq) => RealMetaStructure::Seq(
                mb_seq
                    .into_iter()
                    .map(|mb| into_real_meta_block(mb, map_root_key.as_ref()))
                    .collect()
            ),
            ManyMetaStructure::Map(mb_map) => RealMetaStructure::Map(
                mb_map
                    .into_iter()
                    .map(|(k, mb)| (k, into_real_meta_block(mb, map_root_key.as_ref())))
                    .collect()
            ),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub(crate) enum MetaStructure {
    Unit(UnitMetaStructure),
    Many(ManyMetaStructure),
}

impl MetaStructure {
    pub fn into_real_meta_structure<S: AsRef<str>>(self, map_root_key: S) -> RealMetaStructure {
        match self {
            MetaStructure::Unit(u_ms) => u_ms.into_real_meta_structure(map_root_key),
            MetaStructure::Many(m_ms) => m_ms.into_real_meta_structure(map_root_key),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MetaVal;

    use metadata::types::MetaKey as RealMetaKey;
    use metadata::types::MetaVal as RealMetaVal;

    #[test]
    fn test_into_real_meta_val() {
        const MAP_ROOT_KEY: &str = "key_root";
        let inputs_and_expected = vec![
            (
                MetaVal::Nil,
                RealMetaVal::Nil,
            ),
            (
                MetaVal::Str(String::from("val_a")),
                RealMetaVal::Str(String::from("val_a")),
            ),
            (
                MetaVal::Seq(vec![
                    MetaVal::Str(String::from("val_a")),
                    MetaVal::Str(String::from("val_b")),
                    MetaVal::Str(String::from("val_c")),
                ]),
                RealMetaVal::Seq(vec![
                    RealMetaVal::Str(String::from("val_a")),
                    RealMetaVal::Str(String::from("val_b")),
                    RealMetaVal::Str(String::from("val_c")),
                ]),
            ),
            (
                MetaVal::Map(btreemap![
                    String::from(MAP_ROOT_KEY) => MetaVal::Str(String::from("val_root")),
                    String::from("key_a") => MetaVal::Str(String::from("val_a")),
                    String::from("key_b") => MetaVal::Str(String::from("val_b")),
                ]),
                RealMetaVal::Map(btreemap![
                    RealMetaKey::Nil => RealMetaVal::Str(String::from("val_root")),
                    RealMetaKey::Str(String::from("key_a")) => RealMetaVal::Str(String::from("val_a")),
                    RealMetaKey::Str(String::from("key_b")) => RealMetaVal::Str(String::from("val_b")),
                ]),
            ),
            (
                MetaVal::Map(btreemap![
                    String::from(MAP_ROOT_KEY) => MetaVal::Str(String::from("val_root")),
                    String::from("key_a") => MetaVal::Str(String::from("val_a")),
                    String::from("key_b") => MetaVal::Map(btreemap![
                        String::from(MAP_ROOT_KEY) => MetaVal::Str(String::from("sub_val_root")),
                        String::from("sub_key_a") => MetaVal::Str(String::from("sub_val_a")),
                    ]),
                ]),
                RealMetaVal::Map(btreemap![
                    RealMetaKey::Nil => RealMetaVal::Str(String::from("val_root")),
                    RealMetaKey::Str(String::from("key_a")) => RealMetaVal::Str(String::from("val_a")),
                    RealMetaKey::Str(String::from("key_b")) => RealMetaVal::Map(btreemap![
                        RealMetaKey::Nil => RealMetaVal::Str(String::from("sub_val_root")),
                        RealMetaKey::Str(String::from("sub_key_a")) => RealMetaVal::Str(String::from("sub_val_a")),
                    ]),
                ]),
            ),
            (
                MetaVal::Seq(vec![
                    MetaVal::Str(String::from("val_a")),
                    MetaVal::Map(btreemap![
                        String::from(MAP_ROOT_KEY) => MetaVal::Str(String::from("sub_val_root")),
                        String::from("sub_key_a") => MetaVal::Str(String::from("sub_val_a")),
                    ]),
                ]),
                RealMetaVal::Seq(vec![
                    RealMetaVal::Str(String::from("val_a")),
                    RealMetaVal::Map(btreemap![
                        RealMetaKey::Nil => RealMetaVal::Str(String::from("sub_val_root")),
                        RealMetaKey::Str(String::from("sub_key_a")) => RealMetaVal::Str(String::from("sub_val_a")),
                    ]),
                ]),
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = input.into_real_meta_val(MAP_ROOT_KEY);
            assert_eq!(expected, produced);
        }
    }
}
