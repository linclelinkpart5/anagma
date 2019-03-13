use std::path::Path;

use config::selection::Selection;
use config::sort_order::SortOrder;
use config::meta_format::MetaFormat;
use metadata::types::MetaKey;
use metadata::types::MetaVal;
use metadata::processor::MetaProcessor;
use metadata::processor::Error as ProcessorError;

#[derive(Debug)]
pub enum Error {
    Processor(ProcessorError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::Processor(ref err) => write!(f, "processor error: {}", err),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Self::Processor(ref err) => Some(err),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum FallbackIterKind {
    Parents,
    ChildrenDepth,
    ChildrenBreadth,
}

pub struct PIter<'k, 'p, 's, 'mrk> {
    target_key_path: Vec<&'k MetaKey>,
    next_path: Option<&'p Path>,
    meta_format: MetaFormat,
    selection: &'s Selection,
    sort_order: SortOrder,
    map_root_key: &'mrk str,
}

impl<'k, 'p, 's, 'mrk> Iterator for PIter<'k, 'p, 's, 'mrk> {
    type Item = Result<MetaVal, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_path {
            Some(curr_path) => {
                self.next_path = curr_path.parent();

                let mut processed = MetaProcessor::process_item_file(
                    curr_path,
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
