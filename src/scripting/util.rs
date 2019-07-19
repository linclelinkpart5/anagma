pub mod value_producer;
pub mod iterable_like;
pub mod ref_iterable_like;

use crate::metadata::types::MetaVal;
use crate::scripting::Error;

pub type UnaryConv = fn(MetaVal) -> Result<MetaVal, Error>;
