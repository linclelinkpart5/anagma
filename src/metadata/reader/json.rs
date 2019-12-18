use crate::metadata::structure::MetaStructure;
use crate::metadata::structure::MetaStructureRepr;
use crate::metadata::reader::Error;
use crate::metadata::location::Location;

pub(crate) fn read_str<S: AsRef<str>>(s: S, mt: Location) -> Result<MetaStructure, Error> {
    Ok(match mt {
        Location::Contains => MetaStructureRepr::Unit(serde_json::from_str(s.as_ref()).map_err(Error::JsonDeserializeError)?),
        Location::Siblings => MetaStructureRepr::Many(serde_json::from_str(s.as_ref()).map_err(Error::JsonDeserializeError)?),
    }.into())
}

#[cfg(test)]
mod tests {
    use super::read_str;

    use crate::metadata::location::Location;

    #[test]
    fn test_read_str() {
        let input = r#"
        {
            "key_a": "val_a",
            "key_b": "val_b",
            "key_c": "val_c",
            "key_d": "val_d"
        }
        "#;
        let _ = read_str(input, Location::Contains).unwrap();

        let input = r#"
        {
            "key_a": "val_a",
            "key_b": {
                "sub_key_a": "sub_val_a",
                "sub_key_b": "sub_val_b"
            },
            "key_c": [
                "val_a",
                "val_b"
            ],
            "key_d": {
                "sub_key_a": "sub_val_a",
                "sub_key_b": "sub_val_b"
            },
            "key_e": [
                "val_a",
                "val_b"
            ]
        }
        "#;
        let _ = read_str(input, Location::Contains).unwrap();

        let input = r#"
        [
            {
                "key_1_a": "val_1_a",
                "key_1_b": "val_1_b"
            },
            {
                "key_2_a": "val_2_a",
                "key_2_b": "val_2_b"
            }
        ]
        "#;
        let _ = read_str(input, Location::Siblings).unwrap();

        let input = r#"
        {
            "item_1": {
                "key_1_a": "val_1_a",
                "key_1_b": "val_1_b"
            },
            "item_2": {
                "key_2_a": "val_2_a",
                "key_2_b": "val_2_b"
            }
        }
        "#;
        let _ = read_str(input, Location::Siblings).unwrap();
    }
}
