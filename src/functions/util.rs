pub mod number_like;
pub mod stream_adaptor;

pub use self::number_like::NumberLike;
pub use self::stream_adaptor::StreamAdaptor;
pub use self::stream_adaptor::*;

use crate::metadata::types::MetaVal;
use crate::functions::Error;

/// Namespace for all the implementation of various functions in this module.
pub struct Impl;

impl Impl {
    pub fn collect(sa: StreamAdaptor) -> Result<Vec<MetaVal>, Error> {
        Ok(sa.collect::<Result<Vec<_>, _>>()?)
    }
}
