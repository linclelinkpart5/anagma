pub mod number_like;
pub mod value_producer;
pub mod iterable_like;

pub use self::number_like::NumberLike;
pub use self::value_producer::ValueProducer;
pub use self::value_producer::*;
pub use self::iterable_like::IterableLike;

use crate::metadata::types::MetaVal;
use crate::scripting::Error;

pub type UnaryPred = fn(&MetaVal) -> Result<bool, Error>;
pub type UnaryConv = fn(MetaVal) -> Result<MetaVal, Error>;
