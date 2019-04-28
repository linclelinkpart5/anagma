use std::collections::VecDeque;
use std::collections::HashSet;

use crate::metadata::stream::value::MetaValueStream;
use crate::metadata::stream::value::Error as MetaValueStreamError;
use crate::metadata::types::MetaVal;

#[derive(Debug)]
pub enum Error {
    ValueStream(MetaValueStreamError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::ValueStream(ref err) => write!(f, "value stream error: {}", err),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Self::ValueStream(ref err) => Some(err),
        }
    }
}

#[derive(Debug)]
pub enum StreamAdaptor<'s> {
    Raw(MetaValueStream<'s>),
    Fixed(std::vec::IntoIter<MetaVal<'s>>),

    // Flatten(FlattenStream<'s>),
    // Dedup(DedupStream<'s>),
    // Unique(UniqueStream<'s>),
    // StepBy(StepByStream<'s>),
    // Chain(ChainStream<'s>),
    // Zip(ZipStream<'s>),
    // Map(MapStream<'s>),
}

impl<'s> Iterator for StreamAdaptor<'s> {
    type Item = Result<MetaVal<'s>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            &mut Self::Raw(ref mut it) => it.next().map(|res| res.map(|(_, mv)| mv)).map(|res| res.map_err(Error::ValueStream)),
            &mut Self::Fixed(ref mut it) => it.next().map(Result::Ok),

            // &mut Self::Flatten(ref mut it) => it.next(),
            // &mut Self::Dedup(ref mut it) => it.next(),
            // &mut Self::Unique(ref mut it) => it.next(),
            // &mut Self::StepBy(ref mut it) => it.next(),
            // &mut Self::Chain(ref mut it) => it.next(),
            // &mut Self::Zip(ref mut it) => it.next(),
            // &mut Self::Map(ref mut it) => it.next(),
        }
    }
}
