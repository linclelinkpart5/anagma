pub mod iterable_like;
pub mod number_like;
pub mod streams;
pub mod ops;
pub mod context;
pub mod predicate;

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
    NotIndex,
    NegativeInteger,
    IndexOutOfBounds,
    ZeroStepSize,
    InvalidConverter,
    InvalidPredicate,
    NotUnaryOp,
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
            Self::NotIndex => write!(f, "non index value was found"),
            Self::NegativeInteger => write!(f, "negative indexes are not allowed"),
            Self::IndexOutOfBounds => write!(f, "index is out of bounds"),
            Self::ZeroStepSize => write!(f, "step size must be greater than zero"),
            Self::InvalidConverter => write!(f, "invalid conversion operator"),
            Self::InvalidPredicate => write!(f, "invalid predicate operator"),
            Self::NotUnaryOp => write!(f, "non unary operator value was found"),
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
            Self::NotIndex => None,
            Self::NegativeInteger => None,
            Self::IndexOutOfBounds => None,
            Self::ZeroStepSize => None,
            Self::InvalidConverter => None,
            Self::InvalidPredicate => None,
            Self::NotUnaryOp => None,
        }
    }
}

impl From<ValueStreamError> for Error {
    fn from(err: ValueStreamError) -> Self {
        Self::ValueStream(err)
    }
}
