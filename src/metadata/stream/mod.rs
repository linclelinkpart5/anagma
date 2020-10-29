pub mod block;
pub mod value;

use std::io::Error as IoError;

use thiserror::Error;

use crate::metadata::processor::Error as ProcessorError;

pub use self::block::BlockStream;
pub use self::value::ValueStream;

#[derive(Debug, Error)]
pub enum Error {
    #[error("processor error: {0}")]
    Processor(#[source] ProcessorError),
    #[error("file walker error: {0}")]
    FileWalker(#[source] IoError),
}
