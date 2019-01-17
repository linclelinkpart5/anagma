//! These are helper structs and types used for deserializing metadata.
//! After deserialization, extra post-processing is done to convert these to their final used forms.

use std::collections::BTreeMap;
use std::collections::HashMap;

use metadata::types::MetaVal as RealMetaVal;
use metadata::types::MetaKey as RealMetaKey;

#[derive(PartialEq, Eq, Debug, Clone, Hash, Deserialize)]
pub enum MetaVal {
    Nil,
    Str(String),
    Seq(Vec<MetaVal>),
    Map(BTreeMap<String, MetaVal>),
}

impl MetaVal {
    pub fn to_real_meta_val<S: AsRef<str>>(self, map_root_key: S) -> RealMetaVal {
        match self {
            MetaVal::Nil => RealMetaVal::Nil,
            MetaVal::Str(s) => RealMetaVal::Str(s),
            MetaVal::Seq(seq) => {
                RealMetaVal::Seq(seq.into_iter().map(|mv| mv.to_real_meta_val(map_root_key.as_ref())).collect())
            },
            MetaVal::Map(map) => {
                // All occurences of the map root key must be converted into a null meta key.
                let mut new_map = BTreeMap::new();

                for (k, v) in map {
                    let new_k = match k == map_root_key.as_ref() {
                        true => RealMetaKey::Nil,
                        false => RealMetaKey::Str(k),
                    };

                    let new_v = v.to_real_meta_val(map_root_key.as_ref());

                    new_map.insert(new_k, new_v);
                }

                RealMetaVal::Map(new_map)
            },
        }
    }
}

pub type MetaBlock = BTreeMap<String, MetaVal>;
pub type MetaBlockSeq = Vec<MetaBlock>;
pub type MetaBlockMap = HashMap<String, MetaBlock>;

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub(crate) enum UnitMetaStructure {
    One(MetaBlock),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub(crate) enum ManyMetaStructure {
    Seq(MetaBlockSeq),
    Map(MetaBlockMap),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub(crate) enum MetaStructure {
    Unit(UnitMetaStructure),
    Many(ManyMetaStructure),
}

#[cfg(test)]
mod tests {
    use super::MetaVal;

    use metadata::types::MetaKey as RealMetaKey;
    use metadata::types::MetaVal as RealMetaVal;

    #[test]
    fn test_to_real_meta_val() {
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
            let produced = input.to_real_meta_val(MAP_ROOT_KEY);
            assert_eq!(expected, produced);
        }
    }
}
