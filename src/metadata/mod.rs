//! Primitives and methods for accessing and working with item metadata.

pub mod plexer;
pub mod processor;
pub mod schema;

pub use self::schema::Schema;
pub use self::plexer::{Plexer, Error as PlexerError};
pub use self::processor::Error as ProcessorError;
