use crate::metadata::types::MetaStructure;
use crate::metadata::types::MetaStructureRepr;
use crate::metadata::reader::Error;
use crate::metadata::location::MetaLocation;

pub(crate) fn read_str<S: AsRef<str>>(s: S, mt: MetaLocation) -> Result<MetaStructure, Error> {
    Ok(match mt {
        MetaLocation::Contains => MetaStructureRepr::Unit(serde_yaml::from_str(s.as_ref()).map_err(Error::YamlDeserializeError)?),
        MetaLocation::Siblings => MetaStructureRepr::Many(serde_yaml::from_str(s.as_ref()).map_err(Error::YamlDeserializeError)?),
    }.into())
}

#[cfg(test)]
mod tests {
    use super::read_str;

    use crate::metadata::location::MetaLocation;

    #[test]
    fn test_read_str() {
        let input = r#"
            key_a: val_a
            key_b: val_b
            key_c: val_c
            key_d: val_d
        "#;
        let _ = read_str(input, MetaLocation::Contains).unwrap();

        let input = r#"
            key_a: val_a
            key_b:
                sub_key_a: sub_val_a
                sub_key_b: sub_val_b
            key_c: [val_a, val_b]
            key_d: {sub_key_a: sub_val_a, sub_key_b: sub_val_b}
            key_e:
                -   val_a
                -   val_b
        "#;
        let _ = read_str(input, MetaLocation::Contains).unwrap();

        let input = r#"
            -   key_1_a: val_1_a
                key_1_b: val_1_b
            -   key_2_a: val_2_a
                key_2_b: val_2_b
        "#;
        let _ = read_str(input, MetaLocation::Siblings).unwrap();

        let input = r#"
            item_1:
                key_1_a: val_1_a
                key_1_b: val_1_b
            item_2:
                key_2_a: val_2_a
                key_2_b: val_2_b
        "#;
        let _ = read_str(input, MetaLocation::Siblings).unwrap();
    }
}
