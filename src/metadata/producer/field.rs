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

#[cfg(test)]
mod tests {
    use super::MetaFieldProducer;

    use std::borrow::Cow;
    use std::path::Path;
    use std::collections::VecDeque;
    use test_util::TestUtil;

    use metadata::producer::block::MetaBlockProducer;
    use metadata::producer::block::FixedMetaBlockProducer;
    use metadata::producer::block::FileMetaBlockProducer;
    use metadata::types::MetaKey;
    use metadata::types::MetaVal;
    use config::selection::Selection;
    use config::sort_order::SortOrder;
    use config::meta_format::MetaFormat;
    use util::file_walkers::FileWalker;
    use util::file_walkers::ParentFileWalker;
    use util::file_walkers::ChildFileWalker;

    #[test]
    fn test_meta_field_producer() {
        let temp_dir = TestUtil::create_meta_fanout_test_dir("test_meta_field_producer", 3, 3, TestUtil::default_flag_set_by);
        let root_dir = temp_dir.path();
        let selection = Selection::default();

        let target_file_name_key = MetaKey::from("target_file_name");
        let flag_key = MetaKey::from("flag_key");

        let origin_path = root_dir.join("0").join("0_1").join("0_1_2");
        let file_walker = FileWalker::Parent(ParentFileWalker::new(&origin_path));

        let block_producer = MetaBlockProducer::File(FileMetaBlockProducer::new(
            file_walker,
            MetaFormat::Json,
            &selection,
            SortOrder::Name,
        ));

        let expected = vec![
            (Cow::Owned(root_dir.join("0").join("0_1").join("0_1_2")), MetaVal::from("0_1_2")),
            (Cow::Owned(root_dir.join("0").join("0_1")), MetaVal::from("0_1")),
            (Cow::Owned(root_dir.join("0")), MetaVal::from("0")),
            (Cow::Owned(root_dir.to_path_buf()), MetaVal::from("ROOT")),
        ];
        let produced = {
            MetaFieldProducer::new(vec![&target_file_name_key], block_producer)
                .into_iter()
                .map(|res| res.unwrap())
                .collect::<Vec<_>>()
        };

        assert_eq!(expected, produced);

        let origin_path = root_dir.join("0");
        let file_walker = FileWalker::Child(ChildFileWalker::new(&origin_path));

        let block_producer = MetaBlockProducer::File(FileMetaBlockProducer::new(
            file_walker,
            MetaFormat::Json,
            &selection,
            SortOrder::Name,
        ));

        let expected = vec![
            (Cow::Owned(root_dir.join("0").join("0_0").join("0_0_0")), MetaVal::from("0_0_0")),
            (Cow::Owned(root_dir.join("0").join("0_0").join("0_0_1").join("0_0_1_1")), MetaVal::from("0_0_1_1")),
            (Cow::Owned(root_dir.join("0").join("0_0").join("0_0_2")), MetaVal::from("0_0_2")),
            (Cow::Owned(root_dir.join("0").join("0_1").join("0_1_0")), MetaVal::from("0_1_0")),
            (Cow::Owned(root_dir.join("0").join("0_1").join("0_1_1").join("0_1_1_1")), MetaVal::from("0_1_1_1")),
            (Cow::Owned(root_dir.join("0").join("0_1").join("0_1_2")), MetaVal::from("0_1_2")),
            (Cow::Owned(root_dir.join("0").join("0_2").join("0_2_0")), MetaVal::from("0_2_0")),
            (Cow::Owned(root_dir.join("0").join("0_2").join("0_2_1").join("0_2_1_1")), MetaVal::from("0_2_1_1")),
            (Cow::Owned(root_dir.join("0").join("0_2").join("0_2_2")), MetaVal::from("0_2_2")),
        ];
        let produced = {
            MetaFieldProducer::new(vec![&flag_key], block_producer)
                .into_iter()
                .map(|res| res.unwrap())
                .collect::<Vec<_>>()
        };

        assert_eq!(expected, produced);
    }
}
