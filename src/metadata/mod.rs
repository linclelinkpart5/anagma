//! Primitives and methods for accessing and working with item metadata.

pub mod target;
pub mod plexer;
pub mod processor;
pub mod stream;
pub mod block;
pub mod value;
pub mod schema;

pub use self::value::{Value, Error as ValueError};

pub use self::block::{Block, BlockSequence, BlockMapping};

pub use self::target::{Target, Error as TargetError};

pub use self::processor::Error as ProcessorError;
