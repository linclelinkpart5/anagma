use std::borrow::Cow;
use std::path::Path;

use metadata::types::MetaKey;
use metadata::types::MetaVal;
use metadata::producer::block::MetaBlockProducer;
use metadata::producer::block::Error as MetaBlockProducerError;

#[derive(Debug)]
pub enum Error {
    MetaBlockProducer(MetaBlockProducerError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::MetaBlockProducer(ref err) => write!(f, "meta block producer error: {}", err),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Self::MetaBlockProducer(ref err) => Some(err),
        }
    }
}

pub struct MetaFieldProducer<'k, 'p, 's> {
    target_key_path: Vec<&'k MetaKey>,
    meta_block_producer: MetaBlockProducer<'p, 's>,
}

impl<'k, 'p, 's> MetaFieldProducer<'k, 'p, 's> {
    pub fn new(target_key_path: Vec<&'k MetaKey>, meta_block_producer: MetaBlockProducer<'p, 's>) -> Self {
        Self {
            target_key_path,
            meta_block_producer,
        }
    }
}

impl<'k, 'p, 's> Iterator for MetaFieldProducer<'k, 'p, 's> {
    type Item = Result<(Cow<'p, Path>, MetaVal), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.meta_block_producer.next() {
            Some(mb_res) => {
                match mb_res {
                    Err(err) => Some(Err(Error::MetaBlockProducer(err))),
                    Ok((path, mb)) => {
                        // Initalize the meta value by wrapping the entire meta block in a map.
                        let mut curr_val = MetaVal::Map(mb);

                        match curr_val.resolve_key_path(&self.target_key_path) {
                            // Not found here, delegate to the next iteration.
                            None => {
                                // We need to delve here before proceeding.
                                match self.meta_block_producer.delve() {
                                    Ok(()) => self.next(),
                                    Err(err) => Some(Err(Error::MetaBlockProducer(err))),
                                }
                            },
                            Some(val) => Some(Ok((path, val))),
                        }
                    },
                }
            },
            None => None,
        }
    }
}
