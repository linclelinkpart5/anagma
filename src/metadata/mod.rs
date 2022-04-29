//! Primitives and methods for accessing and working with item metadata.

pub mod item_paths;
pub mod new_schema;
pub mod plexer;
pub mod processor;
pub mod schema;

pub use self::plexer::{Error as PlexerError, Plexer};
pub use self::processor::Error as ProcessorError;
pub use self::schema::{Arity, Schema};

pub use self::new_schema::Metadata;

pub(crate) use self::schema::SchemaRepr;
