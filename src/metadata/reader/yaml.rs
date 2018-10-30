use std::collections::BTreeMap;
// use std::ffi::OsString;

use yaml_rust::Yaml;
use yaml_rust::YamlLoader;
use failure::ResultExt;

use metadata::reader::Error;
use metadata::reader::MetaReader;
use metadata::location::MetaLocation;
use metadata::structure::MetaStructure;
use metadata::types::key::MetaKey;
use metadata::types::val::MetaVal;
use metadata::types::MetaBlock;
use metadata::types::MetaBlockSeq;
use metadata::types::MetaBlockMap;

pub struct YamlMetaReader;

impl MetaReader for YamlMetaReader {
    fn from_str<S: AsRef<str>>(s: S, mt: MetaLocation) -> Result<MetaStructure, Error> {
        let s = s.as_ref();
        let yaml_docs: Vec<Yaml> = YamlLoader::load_from_str(s).map_err(|_| Error::CannotParseMetadata)?;

        if yaml_docs.len() < 1 {
            Err(Error::EmptyMetadata)?
        }

        let yaml_doc = &yaml_docs[0];

        Ok(yaml_as_metadata(yaml_doc, mt)?)
    }
}

// TODO: Encapsulate these functions in an `impl` block.

fn yaml_as_string(y: &Yaml) -> Result<String, Error> {
    const TARGET: &str = "string";

    match *y {
        Yaml::Null => Err(Error::CannotConvert("null", TARGET))?,
        Yaml::Array(_) => Err(Error::CannotConvert("sequence", TARGET))?,
        Yaml::Hash(_) => Err(Error::CannotConvert("mapping", TARGET))?,
        Yaml::String(ref s) => Ok(s.to_string()),

        // TODO: The rest of these need to be revisited.
        // Ideally we would keep them as strings and not convert when parsing.
        Yaml::Real(ref r) => Ok(r.to_string()),
        Yaml::Integer(i) => Ok(i.to_string()),
        Yaml::Boolean(b) => Ok(b.to_string()),
        Yaml::Alias(_) => Err(Error::CannotConvert("alias", TARGET))?,
        Yaml::BadValue => Err(Error::CannotConvert("bad value", TARGET))?,
    }
}

fn yaml_as_meta_key(y: &Yaml) -> Result<MetaKey, Error> {
    match *y {
        Yaml::Null => Ok(MetaKey::Nil),
        _ => Ok(yaml_as_string(y).map(|s| MetaKey::Str(s)).map_err(|_| Error::CannotConvert("YAML", "meta key"))?),
    }
}

fn yaml_as_meta_value(y: &Yaml) -> Result<MetaVal, Error> {
    match *y {
        Yaml::Null => Ok(MetaVal::Nil),
        Yaml::Array(ref arr) => {
            let mut seq: Vec<MetaVal> = vec![];

            // Recursively convert each found YAML item into a meta value.
            for val_y in arr {
                seq.push(yaml_as_meta_value(&val_y)?);
            }

            Ok(MetaVal::Seq(seq))
        },
        Yaml::Hash(ref hsh) => {
            let mut map: BTreeMap<MetaKey, MetaVal> = btreemap![];

            // Recursively convert each found YAML item into a meta value.
            for (key_y, val_y) in hsh {
                let key = yaml_as_meta_key(&key_y)?;
                let val = yaml_as_meta_value(&val_y)?;

                map.insert(key, val);
            }

            Ok(MetaVal::Map(map))
        },
        _ => yaml_as_string(&y).map(|s| MetaVal::Str(s)),
    }
}

fn yaml_as_meta_block(y: &Yaml) -> Result<MetaBlock, Error> {
    // Try to convert to a hash.
    match *y {
        Yaml::Hash(ref hsh) => {
            let mut mb = MetaBlock::new();

            // Keys must be convertible to strings.
            // Values can be any meta value.
            for (key_y, val_y) in hsh {
                let key = yaml_as_string(&key_y)?;
                let val = yaml_as_meta_value(&val_y)?;

                mb.insert(key, val);
            }

            Ok(mb)
        },
        _ => Err(Error::CannotConvert("YAML", "meta block"))?,
    }
}

pub fn yaml_as_meta_block_seq(y: &Yaml) -> Result<MetaBlockSeq, Error> {
    // Try to convert to sequenced item-metadata.
    // We expect a vector of meta blocks.
    match y {
        &Yaml::Array(ref arr) => {
            let mut item_seq = MetaBlockSeq::new();

            for val_y in arr {
                item_seq.push(yaml_as_meta_block(&val_y)?);
            }

            Ok(item_seq)
        },
        _ => Err(Error::CannotConvert("YAML", "meta block sequence"))?,
    }
}

pub fn yaml_as_meta_block_map(y: &Yaml) -> Result<MetaBlockMap, Error> {
    // Try to convert to mapped item-metadata.
    // We expect a mapping of file names to meta blocks.
    match y {
        &Yaml::Hash(ref hsh) => {
            let mut item_map = MetaBlockMap::new();

            for (key_y, val_y) in hsh {
                let key = yaml_as_string(&key_y)?;

                // TODO: Check that key is a valid item name!

                let val = yaml_as_meta_block(&val_y)?;

                item_map.insert(key, val);
            }

            Ok(item_map)
        },
        _ => Err(Error::CannotConvert("YAML", "meta block mapping"))?,
    }
}

pub fn yaml_as_metadata(y: &Yaml, meta_target: MetaLocation) -> Result<MetaStructure, Error> {
    match meta_target {
        MetaLocation::Contains => {
            yaml_as_meta_block(y).map(|m| MetaStructure::One(m))
        },
        MetaLocation::Siblings => {
            yaml_as_meta_block_seq(y).map(|m| MetaStructure::Seq(m))
                .or(yaml_as_meta_block_map(y).map(|m| MetaStructure::Map(m)))
        },
    }
}

#[cfg(test)]
mod tests {
    use metadata::types::key::MetaKey;
    use metadata::types::val::MetaVal;
    use metadata::types::MetaBlock;
    use yaml_rust::YamlLoader;

    use super::{
        yaml_as_string,
        yaml_as_meta_key,
        yaml_as_meta_value,
        yaml_as_meta_block,
    };

    #[test]
    fn test_yaml_as_string() {
        let inputs_and_expected = vec![
            // Strings
            ("foo", Some("foo".to_string())),
            (r#""foo""#, Some("foo".to_string())),
            (r#"'foo'"#, Some("foo".to_string())),
            (r#""\"foo\"""#, Some(r#""foo""#.to_string())),
            (r#""[foo, bar]""#, Some("[foo, bar]".to_string())),
            (r#""foo: bar""#, Some("foo: bar".to_string())),
            (r#""foo:    bar""#, Some("foo:    bar".to_string())),

            // Integers
            ("27", Some("27".to_string())),
            ("-27", Some("-27".to_string())),
            // TODO: This does not work, due to it getting parsed as an int and losing the plus.
            // ("+27", Some("+27".to_string())),

            // Floats
            ("3.14", Some("3.14".to_string())),
            ("3.141592653589793238462643383279502884197", Some("3.141592653589793238462643383279502884197".to_string())),

            // Nulls
            ("~", None),
            ("null", None),

            // Booleans
            ("True", Some("True".to_string())),
            ("true", Some("true".to_string())),
            ("False", Some("False".to_string())),
            ("false", Some("false".to_string())),

            // Sequences
            ("- item_a\n- item_b", None),
            ("- item_a", None),
            ("[item_a, item_b]", None),
            ("[item_a]", None),

            // Mappings
            ("key_a: val_a\nkey_b: val_b", None),
            ("key_a: val_a", None),
            ("{key_a: val_a, key_b: val_b}", None),
            ("{key_a: val_a}", None),

            // Aliases
        ];

        for (input, expected) in inputs_and_expected {
            let yaml = &YamlLoader::load_from_str(input).unwrap()[0];
            let produced = yaml_as_string(yaml).ok();
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_yaml_as_meta_key() {
        let inputs_and_expected = vec![
            // Strings
            ("foo", Some(MetaKey::Str("foo".to_string()))),
            (r#""foo""#, Some(MetaKey::Str("foo".to_string()))),
            (r#"'foo'"#, Some(MetaKey::Str("foo".to_string()))),
            (r#""\"foo\"""#, Some(MetaKey::Str(r#""foo""#.to_string()))),
            (r#""[foo, bar]""#, Some(MetaKey::Str("[foo, bar]".to_string()))),
            (r#""foo: bar""#, Some(MetaKey::Str("foo: bar".to_string()))),
            (r#""foo:    bar""#, Some(MetaKey::Str("foo:    bar".to_string()))),

            // Integers
            ("27", Some(MetaKey::Str("27".to_string()))),
            ("-27", Some(MetaKey::Str("-27".to_string()))),
            // TODO: This does not work, due to it getting parsed as an int and losing the plus.
            // ("+27", Some(MetaKey::Str("+27".to_string()))),

            // Floats
            ("3.14", Some(MetaKey::Str("3.14".to_string()))),
            ("3.141592653589793238462643383279502884197", Some(MetaKey::Str("3.141592653589793238462643383279502884197".to_string()))),

            // Nulls
            ("~", Some(MetaKey::Nil)),
            ("null", Some(MetaKey::Nil)),

            // Booleans
            ("True", Some(MetaKey::Str("True".to_string()))),
            ("true", Some(MetaKey::Str("true".to_string()))),
            ("False", Some(MetaKey::Str("False".to_string()))),
            ("false", Some(MetaKey::Str("false".to_string()))),

            // Sequences
            ("- item_a\n- item_b", None),
            ("- item_a", None),
            ("[item_a, item_b]", None),
            ("[item_a]", None),

            // Mappings
            ("key_a: val_a\nkey_b: val_b", None),
            ("key_a: val_a", None),
            ("{key_a: val_a, key_b: val_b}", None),
            ("{key_a: val_a}", None),

            // Aliases
        ];

        for (input, expected) in inputs_and_expected {
            let yaml = &YamlLoader::load_from_str(input).unwrap()[0];
            let produced = yaml_as_meta_key(yaml).ok();
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_yaml_as_meta_value() {
        let inputs_and_expected = vec![
            // Strings
            ("foo", Some(MetaVal::Str("foo".to_string()))),
            (r#""foo""#, Some(MetaVal::Str("foo".to_string()))),
            (r#"'foo'"#, Some(MetaVal::Str("foo".to_string()))),
            (r#""\"foo\"""#, Some(MetaVal::Str(r#""foo""#.to_string()))),
            (r#""[foo, bar]""#, Some(MetaVal::Str("[foo, bar]".to_string()))),
            (r#""foo: bar""#, Some(MetaVal::Str("foo: bar".to_string()))),
            (r#""foo:    bar""#, Some(MetaVal::Str("foo:    bar".to_string()))),

            // Integers
            ("27", Some(MetaVal::Str("27".to_string()))),
            ("-27", Some(MetaVal::Str("-27".to_string()))),
            // TODO: This does not work, due to it getting parsed as an int and losing the plus.
            // ("+27", Some(MetaVal::Str("+27".to_string()))),

            // Floats
            ("3.14", Some(MetaVal::Str("3.14".to_string()))),
            ("3.141592653589793238462643383279502884197", Some(MetaVal::Str("3.141592653589793238462643383279502884197".to_string()))),

            // Nulls
            ("~", Some(MetaVal::Nil)),
            ("null", Some(MetaVal::Nil)),

            // Booleans
            ("True", Some(MetaVal::Str("True".to_string()))),
            ("true", Some(MetaVal::Str("true".to_string()))),
            ("False", Some(MetaVal::Str("False".to_string()))),
            ("false", Some(MetaVal::Str("false".to_string()))),

            // Sequences
            ("- item_a\n- item_b", Some(MetaVal::Seq(vec![
                MetaVal::Str("item_a".to_string()),
                MetaVal::Str("item_b".to_string()),
            ]))),
            ("- item_a", Some(MetaVal::Seq(vec![
                MetaVal::Str("item_a".to_string()),
            ]))),
            ("[item_a, item_b]", Some(MetaVal::Seq(vec![
                MetaVal::Str("item_a".to_string()),
                MetaVal::Str("item_b".to_string()),
            ]))),
            ("[item_a]", Some(MetaVal::Seq(vec![
                MetaVal::Str("item_a".to_string()),
            ]))),
            ("- 27\n- 42", Some(MetaVal::Seq(vec![
                MetaVal::Str("27".to_string()),
                MetaVal::Str("42".to_string()),
            ]))),
            ("- 27\n- null", Some(MetaVal::Seq(vec![
                MetaVal::Str("27".to_string()),
                MetaVal::Nil,
            ]))),

            // Mappings
            ("key_a: val_a\nkey_b: val_b", Some(MetaVal::Map(btreemap![
                MetaKey::Str("key_a".to_string()) => MetaVal::Str("val_a".to_string()),
                MetaKey::Str("key_b".to_string()) => MetaVal::Str("val_b".to_string()),
            ]))),
            ("key_a: val_a", Some(MetaVal::Map(btreemap![
                MetaKey::Str("key_a".to_string()) => MetaVal::Str("val_a".to_string()),
            ]))),
            ("{key_a: val_a, key_b: val_b}", Some(MetaVal::Map(btreemap![
                MetaKey::Str("key_a".to_string()) => MetaVal::Str("val_a".to_string()),
                MetaKey::Str("key_b".to_string()) => MetaVal::Str("val_b".to_string()),
            ]))),
            ("{key_a: val_a}", Some(MetaVal::Map(btreemap![
                MetaKey::Str("key_a".to_string()) => MetaVal::Str("val_a".to_string()),
            ]))),

            // Aliases
        ];

        for (input, expected) in inputs_and_expected {
            let yaml = &YamlLoader::load_from_str(input).unwrap()[0];
            let produced = yaml_as_meta_value(yaml).ok();
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_yaml_as_meta_block() {
        let inputs_and_expected = vec![
            // Invalid blocks
            ("foo", None),
            ("27", None),
            ("-27", None),
            ("3.14", None),
            ("3.141592653589793238462643383279502884197", None),
            ("~", None),
            ("null", None),
            ("true", None),
            ("false", None),
            ("- item_a\n- item_b", None),
            ("[item_a, item_b]", None),

            // Valid blocks
            ("key_a: val_a\nkey_b: val_b", {
                let mut mb = MetaBlock::new();
                mb.insert("key_a".to_string(), MetaVal::Str("val_a".to_string()));
                mb.insert("key_b".to_string(), MetaVal::Str("val_b".to_string()));
                Some(mb)
            }),
            ("{key_a: val_a, key_b: val_b}", {
                let mut mb = MetaBlock::new();
                mb.insert("key_a".to_string(), MetaVal::Str("val_a".to_string()));
                mb.insert("key_b".to_string(), MetaVal::Str("val_b".to_string()));
                Some(mb)
            }),
            ("{key_a: [val_a_a, val_a_b, val_a_c], key_b: ~}", {
                let mut mb = MetaBlock::new();
                mb.insert(
                    "key_a".to_string(),
                    MetaVal::Seq(vec![
                        MetaVal::Str("val_a_a".to_string()),
                        MetaVal::Str("val_a_b".to_string()),
                        MetaVal::Str("val_a_c".to_string()),
                    ])
                );
                mb.insert("key_b".to_string(), MetaVal::Nil);
                Some(mb)
            }),
            ("{key_a: {sub_key_a: sub_val_a, sub_key_b: sub_val_b, ~: sub_val_c}, key_b: []}", {
                let mut mb = MetaBlock::new();
                mb.insert(
                    "key_a".to_string(),
                    MetaVal::Map(btreemap![
                        MetaKey::Str("sub_key_a".to_string()) => MetaVal::Str("sub_val_a".to_string()),
                        MetaKey::Str("sub_key_b".to_string()) => MetaVal::Str("sub_val_b".to_string()),
                        MetaKey::Nil => MetaVal::Str("sub_val_c".to_string()),
                    ])
                );
                mb.insert("key_b".to_string(), MetaVal::Seq(vec![]));
                Some(mb)
            }),

            // Skipped entries
            // NOTE: As a result of changing from Option to Result, these cases now cause the parsing to fail.
            ("{key_a: val_a, [skipped_key, skipped_key]: skipped_val}", None),
            ("{key_a: val_a, {skipped_key_key: skipped_key_val}: skipped_val}", None),
            ("{key_a: val_a, ~: skipped_val}", None),
            // ("{key_a: val_a, [skipped_key, skipped_key]: skipped_val}", {
            //     let mut mb = MetaBlock::new();
            //     mb.insert("key_a".to_string(), MetaVal::Str("val_a".to_string()));
            //     Some(mb)
            // }),
            // ("{key_a: val_a, {skipped_key_key: skipped_key_val}: skipped_val}", {
            //     let mut mb = MetaBlock::new();
            //     mb.insert("key_a".to_string(), MetaVal::Str("val_a".to_string()));
            //     Some(mb)
            // }),
            // ("{key_a: val_a, ~: skipped_val}", {
            //     let mut mb = MetaBlock::new();
            //     mb.insert("key_a".to_string(), MetaVal::Str("val_a".to_string()));
            //     Some(mb)
            // }),
        ];

        for (input, expected) in inputs_and_expected {
            let yaml = &YamlLoader::load_from_str(input).unwrap()[0];
            let produced = yaml_as_meta_block(yaml).ok();
            assert_eq!(expected, produced);
        }
    }
}
