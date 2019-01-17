use metadata::types::repr::MetaStructure;
use metadata::reader::Error;
use metadata::location::MetaLocation;

pub(crate) fn read_str<S: AsRef<str>>(s: S, mt: MetaLocation) -> Result<MetaStructure, Error> {
    Ok(match mt {
        MetaLocation::Contains => MetaStructure::Unit(serde_yaml::from_str(s.as_ref()).unwrap()),
        MetaLocation::Siblings => MetaStructure::Many(serde_yaml::from_str(s.as_ref()).unwrap()),
    })
}
