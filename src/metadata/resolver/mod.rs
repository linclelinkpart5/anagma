pub mod iterable_like;
pub mod number_like;
pub mod streams;
pub mod ops;
pub mod context;

use crate::metadata::stream::value::Error as ValueStreamError;

#[derive(Debug)]
pub enum Error {
    ValueStream(ValueStreamError),
    UnexpectedOperand,
    EmptyStack,
    NotNumeric,
    EmptyIterable,
    NotString,
    NotIterable,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::ValueStream(ref err) => write!(f, "value stream error: {}", err),
            Self::UnexpectedOperand => write!(f, "unexpected operand on stack"),
            Self::EmptyStack => write!(f, "empty operand stack"),
            Self::NotNumeric => write!(f, "non numeric value was found"),
            Self::EmptyIterable => write!(f, "empty iterable"),
            Self::NotString => write!(f, "non string value was found"),
            Self::NotIterable => write!(f, "non iterable value was found"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Self::ValueStream(ref err) => Some(err),
            Self::UnexpectedOperand => None,
            Self::EmptyStack => None,
            Self::NotNumeric => None,
            Self::EmptyIterable => None,
            Self::NotString => None,
            Self::NotIterable => None,
        }
    }
}

impl From<ValueStreamError> for Error {
    fn from(err: ValueStreamError) -> Self {
        Self::ValueStream(err)
    }
}
