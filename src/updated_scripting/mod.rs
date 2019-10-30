pub mod util;
pub mod traits;
pub mod ops;
pub mod arg;

use crate::metadata::stream::value::Error as MetaValueStreamError;

#[derive(Debug)]
#[cfg_attr(test, derive(EnumDiscriminants))]
#[cfg_attr(test, strum_discriminants(name(ErrorKind)))]
pub enum Error {
    ValueStream(MetaValueStreamError),
    NotNumeric,
    NotSequence,
    NotBoolean,
    NotMapping,
    #[cfg(test)] Sentinel,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::ValueStream(ref err) => write!(f, "value stream error: {}", err),
            Self::NotNumeric => write!(f, "value is not a number"),
            Self::NotSequence => write!(f, "value is not a sequence"),
            Self::NotBoolean => write!(f, "value is not a boolean"),
            Self::NotMapping => write!(f, "value is not a mapping"),
            #[cfg(test)] Self::Sentinel => write!(f, "sentinel error, only for testing"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Self::ValueStream(ref err) => Some(err),
            _ => None,
        }
    }
}
