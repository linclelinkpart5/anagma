pub mod util;
pub mod traits;
pub mod ops;

use crate::metadata::stream::value::Error as MetaValueStreamError;

#[derive(Debug)]
#[cfg_attr(test, derive(EnumDiscriminants))]
#[cfg_attr(test, strum_discriminants(name(ErrorKind)))]
pub enum Error {
    ValueStream(MetaValueStreamError),
    NotNumeric,
    #[cfg(test)] Sentinel,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::ValueStream(ref err) => write!(f, "value stream error: {}", err),
            Self::NotNumeric => write!(f, "value is not a number"),
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
