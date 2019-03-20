//! Iterators that yield meta blocks. This provides a layer of abstraction for later processes that
//! need a stream of meta blocks from various sources.

use std::collections::VecDeque;

use config::selection::Error as SelectionError;
use metadata::types::MetaBlock;

#[derive(Debug)]
pub enum Error {
    Selection(SelectionError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::Selection(ref err) => write!(f, "selection error: {}", err),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Self::Selection(ref err) => Some(err),
        }
    }
}

/// A meta block producer that yields from a fixed sequence.
pub struct FixedMetaBlockProducer(VecDeque<MetaBlock>);

impl Iterator for FixedMetaBlockProducer {
    type Item = Result<MetaBlock, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop_front().map(Result::Ok)
    }
}

pub struct FileMetaBlockProducer;
