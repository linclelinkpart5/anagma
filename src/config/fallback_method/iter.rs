use std::path::Path;

use config::selection::Selection;
use config::sort_order::SortOrder;
use config::meta_format::MetaFormat;
use metadata::types::MetaKey;
use metadata::types::MetaVal;
use metadata::processor::MetaProcessor;
use metadata::processor::Error as ProcessorError;
use util::file_walkers::ParentFileWalker;
use util::file_walkers::ChildrenFileWalker;
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

pub struct ParentIter<'k, 'fw, 's, 'mrk> {
    target_key_path: Vec<&'k MetaKey>,
    file_walker: ParentFileWalker<'fw>,
    meta_format: MetaFormat,
    selection: &'s Selection,
    sort_order: SortOrder,
    map_root_key: &'mrk str,
}

impl<'k, 'fw, 's, 'mrk> Iterator for ParentIter<'k, 'fw, 's, 'mrk> {
    type Item = Result<MetaVal, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.file_walker.next() {
            Some(path) => {
                let mut processed = MetaProcessor::process_item_file(
                    path,
                    self.meta_format,
                    self.selection,
                    self.sort_order,
                    self.map_root_key,
                ).map_err(Error::Processor);

                match processed {
                    Err(err) => Some(Err(err)),
                    Ok(mb) => {
                        // Initalize the meta value by wrapping the entire meta block in a map.
                        let mut curr_val = MetaVal::Map(mb);

                        return match curr_val.resolve_key_path(&self.target_key_path) {
                            // Not found here, delegate to the next iteration.
                            None => self.next(),
                            Some(val) => Some(Ok(val)),
                        };
                    },
                }
            },
            // No more paths to iterate over.
            None => None,
        }
    }
}

pub struct ChildrenIter<'k, 'fw, 's, 'mrk> {
    target_key_path: Vec<&'k MetaKey>,
    file_walker: ChildrenFileWalker<'fw, 's>,
    meta_format: MetaFormat,
    map_root_key: &'mrk str,
}

impl<'k, 'fw, 's, 'mrk> Iterator for ChildrenIter<'k, 'fw, 's, 'mrk> {
    type Item = Result<MetaVal, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.file_walker.next() {
            Some(path_res) => {
                match path_res {
                    Err(err) => Some(Err(Error::FileWalker(err))),
                    Ok(path) => {
                        let mut processed = MetaProcessor::process_item_file(
                            path,
                            self.meta_format,
                            self.file_walker.selection,
                            self.file_walker.sort_order,
                            self.map_root_key,
                        ).map_err(Error::Processor);

                        match processed {
                            Err(err) => Some(Err(err)),
                            Ok(mb) => {
                                // Initalize the meta value by wrapping the entire meta block in a map.
                                let mut curr_val = MetaVal::Map(mb);

                                match curr_val.resolve_key_path(&self.target_key_path) {
                                    // Not found here, delegate to the next iteration.
                                    None => self.next(),
                                    Some(val) => Some(Ok(val)),
                                }
                            },
                        }
                    },
                }
            },
            // No more paths to iterate over.
            None => None,
        }
    }
}
