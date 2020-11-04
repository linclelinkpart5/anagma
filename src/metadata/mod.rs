//! Primitives and methods for accessing and working with item metadata.

pub mod block;
pub mod plexer;
pub mod processor;
pub mod schema;
pub mod value;

pub use self::value::{Value, Error as ValueError};
pub use self::block::{Block, BlockSequence, BlockMapping};
pub use self::schema::Schema;
pub use self::plexer::{Plexer, Error as PlexerError};
pub use self::processor::Error as ProcessorError;
