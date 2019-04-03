use std::borrow::Cow;
use std::path::Path;

use metadata::types::MetaKey;
use metadata::types::MetaVal;
use metadata::stream::block::MetaBlockStream;
use metadata::stream::block::Error as MetaBlockStreamError;

#[derive(Debug)]
pub enum Error {
    MetaBlockStream(MetaBlockStreamError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::MetaBlockStream(ref err) => write!(f, "meta block stream error: {}", err),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Self::MetaBlockStream(ref err) => Some(err),
        }
    }
}

pub struct MetaValueStream<'k, 'p, 's> {
    target_key_path: Vec<&'k MetaKey>,
    meta_block_stream: MetaBlockStream<'p, 's>,
}

impl<'k, 'p, 's> MetaValueStream<'k, 'p, 's> {
    pub fn new(target_key_path: Vec<&'k MetaKey>, meta_block_stream: MetaBlockStream<'p, 's>) -> Self {
        Self {
            target_key_path,
            meta_block_stream,
        }
    }
}

impl<'k, 'p, 's> Iterator for MetaValueStream<'k, 'p, 's> {
    type Item = Result<(Cow<'p, Path>, MetaVal), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.meta_block_stream.next() {
            Some(mb_res) => {
                match mb_res {
                    Err(err) => Some(Err(Error::MetaBlockStream(err))),
                    Ok((path, mb)) => {
                        // Initalize the meta value by wrapping the entire meta block in a map.
                        let mut curr_val = MetaVal::Map(mb);

                        match curr_val.resolve_key_path(&self.target_key_path) {
                            // Not found here, delegate to the next iteration.
                            None => {
                                // We need to delve here before proceeding.
                                match self.meta_block_stream.delve() {
                                    Ok(()) => self.next(),
                                    Err(err) => Some(Err(Error::MetaBlockStream(err))),
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

/// A convenience newtype that only yields meta values, and logs and skips errors.
pub struct SimpleMetaValueStream<'k, 'p, 's>(MetaValueStream<'k, 'p, 's>);

impl<'k, 'p, 's> Iterator for SimpleMetaValueStream<'k, 'p, 's> {
    type Item = MetaVal;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.next() {
            None => None,
            Some(Ok((_, mv))) => Some(mv),
            Some(Err(err)) => {
                warn!("{}", err);
                self.next()
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MetaValueStream;

    use std::borrow::Cow;
    use std::path::Path;
    use std::collections::VecDeque;
    use test_util::TestUtil;

    use metadata::stream::block::MetaBlockStream;
    use metadata::stream::block::FixedMetaBlockStream;
    use metadata::stream::block::FileMetaBlockStream;
    use metadata::types::MetaKey;
    use metadata::types::MetaVal;
    use config::selection::Selection;
    use config::sort_order::SortOrder;
    use config::meta_format::MetaFormat;
    use util::file_walkers::FileWalker;
    use util::file_walkers::ParentFileWalker;
    use util::file_walkers::ChildFileWalker;

    #[test]
    fn test_meta_field_stream_all() {
        let temp_dir = TestUtil::create_meta_fanout_test_dir("test_meta_field_stream_all", 3, 3, TestUtil::flag_set_by_all);
        let root_dir = temp_dir.path();
        let selection = Selection::default();

        let target_key = MetaKey::from("flag_key");

        let origin_path = root_dir.join("0").join("0_1").join("0_1_2");
        let file_walker = FileWalker::Parent(ParentFileWalker::new(&origin_path));

        let block_stream = MetaBlockStream::File(FileMetaBlockStream::new(
            file_walker,
            MetaFormat::Json,
            &selection,
            SortOrder::Name,
        ));

        let expected = vec![
            (Cow::Owned(root_dir.join("0").join("0_1").join("0_1_2")), MetaVal::from("0_1_2")),
            (Cow::Owned(root_dir.join("0").join("0_1")), MetaVal::from("0_1")),
            (Cow::Owned(root_dir.join("0")), MetaVal::from("0")),
            // (Cow::Owned(root_dir.to_path_buf()), MetaVal::from("ROOT")),
        ];
        let produced = {
            MetaValueStream::new(vec![&target_key], block_stream)
                .into_iter()
                .map(|res| res.unwrap())
                .collect::<Vec<_>>()
        };

        assert_eq!(expected, produced);
    }

    #[test]
    fn test_meta_field_stream_default() {
        let temp_dir = TestUtil::create_meta_fanout_test_dir("test_meta_field_stream_default", 3, 3, TestUtil::flag_set_by_default);
        let root_dir = temp_dir.path();
        let selection = Selection::default();

        let target_key = MetaKey::from("flag_key");

        let origin_path = root_dir.join("0").join("0_1").join("0_1_2");
        let file_walker = FileWalker::Parent(ParentFileWalker::new(&origin_path));

        let block_stream = MetaBlockStream::File(FileMetaBlockStream::new(
            file_walker,
            MetaFormat::Json,
            &selection,
            SortOrder::Name,
        ));

        let expected = vec![
            (Cow::Owned(root_dir.join("0").join("0_1").join("0_1_2")), MetaVal::from("0_1_2")),
        ];
        let produced = {
            MetaValueStream::new(vec![&target_key], block_stream)
                .into_iter()
                .map(|res| res.unwrap())
                .collect::<Vec<_>>()
        };

        assert_eq!(expected, produced);

        let origin_path = root_dir.join("0");
        let file_walker = FileWalker::Child(ChildFileWalker::new(&origin_path));

        let block_stream = MetaBlockStream::File(FileMetaBlockStream::new(
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
            MetaValueStream::new(vec![&target_key], block_stream)
                .into_iter()
                .map(|res| res.unwrap())
                .collect::<Vec<_>>()
        };

        assert_eq!(expected, produced);
    }

    #[test]
    fn test_meta_field_stream_none() {
        let temp_dir = TestUtil::create_meta_fanout_test_dir("test_meta_field_stream_none", 3, 3, TestUtil::flag_set_by_none);
        let root_dir = temp_dir.path();
        let selection = Selection::default();

        let target_key = MetaKey::from("flag_key");

        let origin_path = root_dir.join("0").join("0_1").join("0_1_2");
        let file_walker = FileWalker::Parent(ParentFileWalker::new(&origin_path));

        let block_stream = MetaBlockStream::File(FileMetaBlockStream::new(
            file_walker,
            MetaFormat::Json,
            &selection,
            SortOrder::Name,
        ));

        let expected: Vec<(Cow<'_, _>, MetaVal)> = vec![];
        let produced = {
            MetaValueStream::new(vec![&target_key], block_stream)
                .into_iter()
                .map(|res| res.unwrap())
                .collect::<Vec<_>>()
        };

        assert_eq!(expected, produced);

        let origin_path = root_dir.join("0");
        let file_walker = FileWalker::Child(ChildFileWalker::new(&origin_path));

        let block_stream = MetaBlockStream::File(FileMetaBlockStream::new(
            file_walker,
            MetaFormat::Json,
            &selection,
            SortOrder::Name,
        ));

        let expected: Vec<(Cow<'_, _>, MetaVal)> = vec![];
        let produced = {
            MetaValueStream::new(vec![&target_key], block_stream)
                .into_iter()
                .map(|res| res.unwrap())
                .collect::<Vec<_>>()
        };

        assert_eq!(expected, produced);
    }
}
