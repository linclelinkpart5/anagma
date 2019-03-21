//! Iterators that yield meta blocks. This provides a layer of abstraction for later processes that
//! need a stream of meta blocks from various sources.

use std::collections::VecDeque;

use config::selection::Selection;
use config::sort_order::SortOrder;
use config::meta_format::MetaFormat;
use metadata::types::MetaBlock;
use metadata::processor::MetaProcessor;
use metadata::processor::Error as ProcessorError;
use util::file_walkers::FileWalker;
use util::file_walkers::Error as FileWalkerError;

#[derive(Debug)]
pub enum Error {
    Processor(ProcessorError),
    FileWalker(FileWalkerError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::Processor(ref err) => write!(f, "processor error: {}", err),
            Self::FileWalker(ref err) => write!(f, "file walker error: {}", err),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Self::Processor(ref err) => Some(err),
            Self::FileWalker(ref err) => Some(err),
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

pub struct FileMetaBlockProducer<'p, 's, 'mrk> {
    file_walker: FileWalker<'p, 's>,
    meta_format: MetaFormat,
    selection: &'s Selection,
    sort_order: SortOrder,
    map_root_key: &'mrk str,
}

impl<'p, 's, 'mrk> Iterator for FileMetaBlockProducer<'p, 's, 'mrk> {
    type Item = Result<MetaBlock, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.file_walker.next() {
            Some(path_res) => {
                match path_res {
                    Ok(path) => {
                        let mut processed = MetaProcessor::process_item_file(
                            &path,
                            self.meta_format,
                            self.selection,
                            self.sort_order,
                            self.map_root_key,
                        ).map_err(Error::Processor);

                        Some(processed)
                    },
                    Err(err) => Some(Err(Error::FileWalker(err))),
                }
            },
            None => None,
        }
    }
}

impl<'p, 's, 'mrk> FileMetaBlockProducer<'p, 's, 'mrk> {
    pub fn delve(&mut self) -> Result<(), Error> {
        self.file_walker.delve().map_err(Error::FileWalker)
    }
}
