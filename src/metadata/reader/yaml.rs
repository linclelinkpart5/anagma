use crate::metadata::structure::MetaStructure;
use crate::metadata::structure::MetaStructureRepr;
use crate::metadata::reader::Error;
use crate::metadata::target::Target;

pub(crate) fn read_str<S: AsRef<str>>(s: S, mt: Target) -> Result<MetaStructure, Error> {
    Ok(match mt {
        Target::Parent => MetaStructureRepr::Unit(serde_yaml::from_str(s.as_ref()).map_err(Error::YamlDeserializeError)?),
        Target::Siblings => MetaStructureRepr::Many(serde_yaml::from_str(s.as_ref()).map_err(Error::YamlDeserializeError)?),
    }.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_str() {
        let input = r#"
            key_a: val_a
            key_b: val_b
            key_c: val_c
            key_d: val_d
        "#;
        assert_matches!(read_str(input, Target::Parent), Ok(MetaStructure::One(_)));

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
        assert_matches!(read_str(input, Target::Parent), Ok(MetaStructure::One(_)));

        let input = r#"
            -   key_1_a: val_1_a
                key_1_b: val_1_b
            -   key_2_a: val_2_a
                key_2_b: val_2_b
        "#;
        assert_matches!(read_str(input, Target::Siblings), Ok(MetaStructure::Seq(_)));

        let input = r#"
            item_1:
                key_1_a: val_1_a
                key_1_b: val_1_b
            item_2:
                key_2_a: val_2_a
                key_2_b: val_2_b
        "#;
        assert_matches!(read_str(input, Target::Siblings), Ok(MetaStructure::Map(_)));
    }
}
