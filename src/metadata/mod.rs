//! Primitives and methods for accessing and working with item metadata.

pub mod value;
pub mod block;
pub mod schema;
pub mod target;
pub mod plexer;
pub mod processor;
pub mod stream;

pub use self::value::{Value, Error as ValueError};
pub use self::block::{Block, BlockSequence, BlockMapping};
pub use self::schema::Schema;
pub use self::target::{Target, Error as TargetError};
pub use self::plexer::{Plexer, Error as PlexerError};
pub use self::processor::Error as ProcessorError;
pub use self::stream::{BlockStream, ValueStream, Error as StreamError};
