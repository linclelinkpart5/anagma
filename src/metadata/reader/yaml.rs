use std::collections::BTreeMap;

use yaml_rust::Yaml;
use yaml_rust::YamlLoader;
use failure::Error;

use metadata::reader::MetaReader;
use metadata::target::MetaTarget;
use metadata::types::Metadata;
use metadata::types::MetaKey;
use metadata::types::MetaValue;

pub struct YamlMetaReader;

impl MetaReader for YamlMetaReader {
    fn from_str<S: AsRef<str>>(s: S, mt: MetaTarget) -> Result<Metadata, Error> {
        let s = s.as_ref();
        let yaml_docs: Vec<Yaml> = YamlLoader::load_from_str(s)?;

        ensure!(yaml_docs.len() >= 1, "empty YAML document");

        let yaml_doc = &yaml_docs[0];

        bail!("NOT GOOD")

        // yaml_as_metadata(yaml_doc, mt)
    }
}

fn yaml_as_string(y: &Yaml) -> Result<String, Error> {
    match *y {
        Yaml::Null => bail!("cannot convert null to string"),
        Yaml::Array(_) => bail!("cannot convert sequence to string"),
        Yaml::Hash(_) => bail!("cannot convert mapping to string"),
        Yaml::String(ref s) => Ok(s.to_string()),

        // TODO: The rest of these need to be revisited.
        // Ideally we would keep them as strings and not convert when parsing.
        Yaml::Real(ref r) => Ok(r.to_string()),
        Yaml::Integer(i) => Ok(i.to_string()),
        Yaml::Boolean(b) => Ok(b.to_string()),
        Yaml::Alias(_) => bail!("cannot convert alias to string"),
        Yaml::BadValue => bail!("cannot convert bad value to string"),
    }
}

fn yaml_as_meta_key(y: &Yaml) -> Result<MetaKey, Error> {
    match *y {
        Yaml::Null => Ok(MetaKey::Nil),
        // _ => yaml_as_string(y).map(|s| MetaKey::Str(s)).chain_err(|| "cannot convert YAML to meta key"),
        _ => yaml_as_string(y).map(|s| MetaKey::Str(s)),
    }
}

fn yaml_as_meta_value(y: &Yaml) -> Result<MetaValue, Error> {
    match *y {
        Yaml::Null => Ok(MetaValue::Nil),
        Yaml::Array(ref arr) => {
            let mut seq: Vec<MetaValue> = vec![];

            // Recursively convert each found YAML item into a meta value.
            for val_y in arr {
                seq.push(yaml_as_meta_value(&val_y)?);
            }

            Ok(MetaValue::Seq(seq))
        },
        Yaml::Hash(ref hsh) => {
            let mut map: BTreeMap<MetaKey, MetaValue> = btreemap![];

            // Recursively convert each found YAML item into a meta value.
            for (key_y, val_y) in hsh {
                let key = yaml_as_meta_key(&key_y)?;
                let val = yaml_as_meta_value(&val_y)?;

                map.insert(key, val);
            }

            Ok(MetaValue::Map(map))
        },
        // _ => yaml_as_string(&y).map(|s| MetaValue::Str(s)).chain_err(|| "cannot convert YAML to meta value"),
        _ => yaml_as_string(&y).map(|s| MetaValue::Str(s)),
    }
}
