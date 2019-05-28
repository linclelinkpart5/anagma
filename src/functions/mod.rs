pub mod operand;
pub mod operator;
pub mod util;

use crate::metadata::stream::value::Error as MetaValueStreamError;

#[derive(Debug)]
pub enum Error {
    ValueStream(MetaValueStreamError),
    EmptyStack,
    NotIterable,
    NotSequence,
    NotNumeric,
    NotPredicate,
    NotConverter,
    NotIndex,
    InvalidOperand,
    EmptySequence,
    EmptyStream,
    EmptyIterable,
    ZeroStepSize,
    OutOfBounds,
    ItemNotFound,
    #[cfg(test)] Sentinel,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::ValueStream(ref err) => write!(f, "value stream error: {}", err),
            Self::EmptyStack => write!(f, "empty operand stack"),
            Self::NotIterable => write!(f, "not an iterable"),
            Self::NotSequence => write!(f, "not a sequence"),
            Self::NotNumeric => write!(f, "not a number"),
            Self::NotPredicate => write!(f, "not a predicate"),
            Self::NotConverter => write!(f, "not a converter"),
            Self::NotIndex => write!(f, "not an index"),
            Self::InvalidOperand => write!(f, "invalid operand"),
            Self::EmptySequence => write!(f, "empty sequence"),
            Self::EmptyStream => write!(f, "empty stream"),
            Self::EmptyIterable => write!(f, "empty iterable"),
            Self::ZeroStepSize => write!(f, "zero step size"),
            Self::OutOfBounds => write!(f, "index out of bounds"),
            Self::ItemNotFound => write!(f, "item not found"),
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
