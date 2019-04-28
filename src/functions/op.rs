pub mod operand;
pub mod operator;

#[derive(Debug)]
pub enum Error {
    EmptyStack,
    NotIterable,
    NotSequence,
    InvalidOperand,
    EmptySequence,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::EmptyStack => write!(f, "empty operand stack"),
            Self::NotIterable => write!(f, "not an iterable"),
            Self::NotSequence => write!(f, "not a sequence"),
            Self::InvalidOperand => write!(f, "invalid operand"),
            Self::EmptySequence => write!(f, "empty sequence"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Self::EmptyStack => None,
            Self::NotIterable => None,
            Self::NotSequence => None,
            Self::InvalidOperand => None,
            Self::EmptySequence => None,
        }
    }
}
