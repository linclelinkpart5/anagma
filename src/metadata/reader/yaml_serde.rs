use metadata::types::repr::MetaStructure;
use metadata::reader::Error;
use metadata::location::MetaLocation;

pub(crate) fn read_str<S: AsRef<str>>(s: S, mt: MetaLocation) -> Result<MetaStructure, Error> {
    Ok(match mt {
        MetaLocation::Contains => MetaStructure::Unit(serde_yaml::from_str(s.as_ref()).map_err(Error::YamlDeserializeError)?),
        MetaLocation::Siblings => MetaStructure::Many(serde_yaml::from_str(s.as_ref()).map_err(Error::YamlDeserializeError)?),
    })
}

#[cfg(test)]
mod tests {
    use super::read_str;

    use metadata::location::MetaLocation;

    #[test]
    fn test_read_str() {
        let input = r#"const_key: const_val
self_key: self_val
ROOT_self_key: ROOT_self_val
overridden: ROOT_self
"#;
        let ms_repr = read_str(input, MetaLocation::Contains).unwrap();
    }
}
