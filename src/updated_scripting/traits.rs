use crate::metadata::types::MetaVal;
use crate::updated_scripting::Error;

/// Trait for types that can take a reference to a meta value and return a boolean.
/// All predicates can also function as converters.
pub trait Predicate: Converter {
    fn test(&self, mv: &MetaVal) -> Result<bool, Error>;

    fn convert(&self, mv: MetaVal) -> Result<MetaVal, Error> {
        Ok(MetaVal::Bul(self.test(&mv)?))
    }
}

/// Trait for types that can take ownership of a meta value and transform it into another meta value.
pub trait Converter {
    fn convert(&self, mv: MetaVal) -> Result<MetaVal, Error>;
}
