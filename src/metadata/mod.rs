//! Primitives and methods for accessing and working with item metadata.

pub mod plexer;
pub mod processor;
pub mod schema;

pub use self::schema::{Arity, Schema};
pub use self::plexer::{Plexer, Error as PlexerError};
pub use self::processor::Error as ProcessorError;

pub(crate) use self::schema::SchemaRepr;
