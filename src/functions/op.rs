pub mod operand;
pub mod operator;

use crate::metadata::stream::value::Error as MetaValueStreamError;

#[derive(Debug)]
pub enum Error {
    ValueStream(MetaValueStreamError),
    EmptyStack,
    NotIterable,
    NotSequence,
    InvalidOperand,
    EmptySequence,
    EmptyStream,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::ValueStream(ref err) => write!(f, "value stream error: {}", err),
            Self::EmptyStack => write!(f, "empty operand stack"),
            Self::NotIterable => write!(f, "not an iterable"),
            Self::NotSequence => write!(f, "not a sequence"),
            Self::InvalidOperand => write!(f, "invalid operand"),
            Self::EmptySequence => write!(f, "empty sequence"),
            Self::EmptyStream => write!(f, "empty stream"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Self::ValueStream(ref err) => Some(err),
            Self::EmptyStack => None,
            Self::NotIterable => None,
            Self::NotSequence => None,
            Self::InvalidOperand => None,
            Self::EmptySequence => None,
            Self::EmptyStream => None,
        }
    }
}
