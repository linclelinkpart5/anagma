pub mod expr;
pub mod util;

use crate::metadata::stream::value::Error as MetaValueStreamError;

#[derive(Debug)]
#[cfg_attr(test, derive(EnumDiscriminants))]
#[cfg_attr(test, strum_discriminants(name(ErrorKind)))]
pub enum Error {
    ValueStream(MetaValueStreamError),
    EmptyStack,
    NotSequence,
    NotProducer,
    NotNumeric,
    NotPredicate,
    NotConverter,
    NotIterable,
    NotBoolean,
    NotUsize,
    NotExpression,
    EmptySequence,
    EmptyProducer,
    ZeroStepSize,
    OutOfBounds,
    ItemNotFound,
    #[cfg(test)] Sentinel,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::ValueStream(ref err) => write!(f, "value stream error: {}", err),
            Self::EmptyStack => write!(f, "empty arg stack"),
            Self::NotSequence => write!(f, "not a sequence"),
            Self::NotProducer => write!(f, "not a producer"),
            Self::NotNumeric => write!(f, "not a number"),
            Self::NotPredicate => write!(f, "not a predicate"),
            Self::NotConverter => write!(f, "not a converter"),
            Self::NotIterable => write!(f, "not an iterable"),
            Self::NotBoolean => write!(f, "not a boolean"),
            Self::NotUsize => write!(f, "not a usize"),
            Self::NotExpression => write!(f, "not an expression"),
            Self::EmptySequence => write!(f, "empty sequence"),
            Self::EmptyProducer => write!(f, "empty producer"),
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
