pub mod block;
pub mod value;

use std::io::Error as IoError;

use crate::metadata::processor::Error as ProcessorError;

pub use self::block::BlockStream;
pub use self::value::ValueStream;

#[derive(Debug)]
pub enum Error {
    Processor(ProcessorError),
    FileWalker(IoError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::Processor(ref err) => write!(f, "processor error: {}", err),
            Self::FileWalker(ref err) => write!(f, "file walker error: {}", err),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Self::Processor(ref err) => Some(err),
            Self::FileWalker(ref err) => Some(err),
        }
    }
}
