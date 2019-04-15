use std::borrow::Cow;
use std::path::Path;
use std::collections::VecDeque;

use crate::metadata::types::MetaKeyPath;
use crate::metadata::types::MetaVal;
use crate::metadata::stream::block::MetaBlockStream;
use crate::metadata::stream::block::Error as MetaBlockStreamError;

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

#[derive(Debug)]
pub enum MetaValueStream<'mvs> {
    Fixed(FixedMetaValueStream<'mvs>),
    Block(BlockMetaValueStream<'mvs>),
}

impl<'mvs> Iterator for MetaValueStream<'mvs> {
    type Item = Result<(Cow<'mvs, Path>, MetaVal<'mvs>), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            &mut Self::Fixed(ref mut it) => it.next(),
            &mut Self::Block(ref mut it) => it.next(),
        }
    }
}

impl<'mvs> From<FixedMetaValueStream<'mvs>> for MetaValueStream<'mvs> {
    fn from(other: FixedMetaValueStream<'mvs>) -> Self {
        Self::Fixed(other)
    }
}

impl<'mvs> From<BlockMetaValueStream<'mvs>> for MetaValueStream<'mvs> {
    fn from(other: BlockMetaValueStream<'mvs>) -> Self {
        Self::Block(other)
    }
}

#[derive(Debug)]
pub struct FixedMetaValueStream<'mvs>(VecDeque<(Cow<'mvs, Path>, MetaVal<'mvs>)>);

impl<'mvs> FixedMetaValueStream<'mvs> {
    pub fn new<II>(items: II) -> Self
    where
        II: IntoIterator<Item = (Cow<'mvs, Path>, MetaVal<'mvs>)>,
    {
        FixedMetaValueStream(items.into_iter().collect())
    }
}

impl<'mvs> Iterator for FixedMetaValueStream<'mvs> {
    type Item = Result<(Cow<'mvs, Path>, MetaVal<'mvs>), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop_front().map(Result::Ok)
    }
}

#[derive(Debug)]
pub struct BlockMetaValueStream<'mvs> {
    target_key_path: MetaKeyPath<'mvs>,
    meta_block_stream: MetaBlockStream<'mvs>,
}

impl<'mvs> BlockMetaValueStream<'mvs> {
    pub fn new<MBS>(target_key_path: MetaKeyPath<'mvs>, meta_block_stream: MBS) -> Self
    where
        MBS: Into<MetaBlockStream<'mvs>>,
    {
        Self {
            target_key_path,
            meta_block_stream: meta_block_stream.into(),
        }
    }
}

impl<'mvs> Iterator for BlockMetaValueStream<'mvs> {
    type Item = Result<(Cow<'mvs, Path>, MetaVal<'mvs>), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.meta_block_stream.next() {
            Some(mb_res) => {
                match mb_res {
                    Err(err) => Some(Err(Error::MetaBlockStream(err))),
                    Ok((path, mb)) => {
                        // Initalize the meta value by wrapping the entire meta block in a map.
                        let curr_val = MetaVal::Map(mb);

                        match curr_val.get_key_path(&self.target_key_path) {
                            // Not found here, delegate to the next iteration.
                            None => {
                                // We need to delve here before proceeding.
                                match self.meta_block_stream.delve() {
                                    Ok(()) => self.next(),
                                    Err(err) => Some(Err(Error::MetaBlockStream(err))),
                                }
                            },
                            Some(val) => Some(Ok((path, val.clone()))),
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
    use super::BlockMetaValueStream;

    use std::borrow::Cow;
    use crate::test_util::TestUtil;

    use crate::metadata::stream::block::MetaBlockStream;
    use crate::metadata::stream::block::FileMetaBlockStream;

    use crate::metadata::types::MetaKeyPath;
    use crate::metadata::types::MetaVal;
    use crate::config::selection::Selection;
    use crate::config::sort_order::SortOrder;
    use crate::config::meta_format::MetaFormat;
    use crate::util::file_walkers::FileWalker;
    use crate::util::file_walkers::ParentFileWalker;
    use crate::util::file_walkers::ChildFileWalker;

    #[test]
    fn test_meta_field_stream_all() {
        let temp_dir = TestUtil::create_meta_fanout_test_dir("test_meta_field_stream_all", 3, 3, TestUtil::flag_set_by_all);
        let root_dir = temp_dir.path();
        let selection = Selection::default();

        let origin_path = root_dir.join("0").join("0_1").join("0_1_2");
        let file_walker = FileWalker::Parent(ParentFileWalker::new(&origin_path));

        let target_key_path = MetaKeyPath::from("flag_key");

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
            BlockMetaValueStream::new(target_key_path.clone(), block_stream)
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

        let origin_path = root_dir.join("0").join("0_1").join("0_1_2");
        let file_walker = FileWalker::Parent(ParentFileWalker::new(&origin_path));

        let target_key_path = MetaKeyPath::from("flag_key");

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
            BlockMetaValueStream::new(target_key_path.clone(), block_stream)
                .into_iter()
                .map(|res| res.unwrap())
                .collect::<Vec<_>>()
        };

        assert_eq!(expected, produced);

        let origin_path = root_dir.join("0");
        let file_walker = FileWalker::Child(ChildFileWalker::new(&origin_path));

        let target_key_path = MetaKeyPath::from("flag_key");

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
            BlockMetaValueStream::new(target_key_path.clone(), block_stream)
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

        let origin_path = root_dir.join("0").join("0_1").join("0_1_2");
        let file_walker = FileWalker::Parent(ParentFileWalker::new(&origin_path));

        let target_key_path = MetaKeyPath::from("flag_key");

        let block_stream = MetaBlockStream::File(FileMetaBlockStream::new(
            file_walker,
            MetaFormat::Json,
            &selection,
            SortOrder::Name,
        ));

        let expected: Vec<(Cow<'_, _>, MetaVal)> = vec![];
        let produced = {
            BlockMetaValueStream::new(target_key_path.clone(), block_stream)
                .into_iter()
                .map(|res| res.unwrap())
                .collect::<Vec<_>>()
        };

        assert_eq!(expected, produced);

        let origin_path = root_dir.join("0");
        let file_walker = FileWalker::Child(ChildFileWalker::new(&origin_path));

        let target_key_path = MetaKeyPath::from("flag_key");

        let block_stream = MetaBlockStream::File(FileMetaBlockStream::new(
            file_walker,
            MetaFormat::Json,
            &selection,
            SortOrder::Name,
        ));

        let expected: Vec<(Cow<'_, _>, MetaVal)> = vec![];
        let produced = {
            BlockMetaValueStream::new(target_key_path.clone(), block_stream)
                .into_iter()
                .map(|res| res.unwrap())
                .collect::<Vec<_>>()
        };

        assert_eq!(expected, produced);
    }
}
