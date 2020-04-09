use std::borrow::Cow;
use std::path::Path;

use super::Error;

use crate::metadata::value::Value;
use crate::metadata::stream::block::BlockStream;

#[derive(Debug)]
pub struct ValueStream<'p> {
    target_key_path: Vec<String>,
    block_stream: BlockStream<'p>,
}

impl<'p> ValueStream<'p> {
    pub fn new(target_key_path: Vec<String>, block_stream: BlockStream<'p>) -> Self {
        Self { target_key_path, block_stream, }
    }
}

impl<'p> Iterator for ValueStream<'p> {
    type Item = Result<(Cow<'p, Path>, Value), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.block_stream.next() {
            Some(mb_res) => {
                match mb_res {
                    Err(err) => Some(Err(err)),
                    Ok((path, mb)) => {
                        // Initalize the meta value by wrapping the entire meta block in a map.
                        // Having metadata keys be simple strings makes this easy and possible!
                        let curr_val = Value::Mapping(mb);

                        match curr_val.get_key_path(&self.target_key_path) {
                            // Not found here, delegate to the next iteration.
                            None => {
                                // We need to delve here before proceeding.
                                match self.block_stream.delve() {
                                    Ok(()) => self.next(),
                                    Err(err) => Some(Err(err)),
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
    use super::ValueStream;

    use std::borrow::Cow;
    use crate::test_util::TestUtil;

    use crate::metadata::stream::block::BlockStream;

    use crate::metadata::value::Value;
    use crate::config::selection::Selection;
    use crate::config::sorter::Sorter;
    use crate::metadata::schema::SchemaFormat;
    use crate::util::file_walker::FileWalker;
    use crate::util::file_walker::ParentFileWalker;
    use crate::util::file_walker::ChildFileWalker;

    #[test]
    fn meta_field_stream_all() {
        let temp_dir = TestUtil::create_meta_fanout_test_dir("meta_field_stream_all", 3, 3, TestUtil::flag_set_by_all);
        let root_dir = temp_dir.path();
        let selection = Selection::default();

        let origin_path = root_dir.join("0").join("0_1").join("0_1_2");
        let file_walker = FileWalker::Parent(ParentFileWalker::new(&origin_path));

        let target_key_path = vec![String::from("flag_key")];

        let block_stream = BlockStream::new(
            file_walker,
            SchemaFormat::Json,
            &selection,
            Sorter::default(),
        );

        let expected = vec![
            (Cow::Owned(root_dir.join("0").join("0_1").join("0_1_2")), Value::from("0_1_2")),
            (Cow::Owned(root_dir.join("0").join("0_1")), Value::from("0_1")),
            (Cow::Owned(root_dir.join("0")), Value::from("0")),
            // (Cow::Owned(root_dir.to_path_buf()), Value::from("ROOT")),
        ];
        let produced = {
            ValueStream::new(target_key_path.clone(), block_stream)
                .into_iter()
                .map(|res| res.unwrap())
                .collect::<Vec<_>>()
        };

        assert_eq!(expected, produced);
    }

    #[test]
    fn meta_field_stream_default() {
        let temp_dir = TestUtil::create_meta_fanout_test_dir("meta_field_stream_default", 3, 3, TestUtil::flag_set_by_default);
        let root_dir = temp_dir.path();
        let selection = Selection::default();

        let origin_path = root_dir.join("0").join("0_1").join("0_1_2");
        let file_walker = FileWalker::Parent(ParentFileWalker::new(&origin_path));

        let target_key_path = vec![String::from("flag_key")];

        let block_stream = BlockStream::new(
            file_walker,
            SchemaFormat::Json,
            &selection,
            Sorter::default(),
        );

        let expected = vec![
            (Cow::Owned(root_dir.join("0").join("0_1").join("0_1_2")), Value::from("0_1_2")),
        ];
        let produced = {
            ValueStream::new(target_key_path.clone(), block_stream)
                .into_iter()
                .map(|res| res.unwrap())
                .collect::<Vec<_>>()
        };

        assert_eq!(expected, produced);

        let origin_path = root_dir.join("0");
        let file_walker = FileWalker::Child(ChildFileWalker::new(&origin_path));

        let target_key_path = vec![String::from("flag_key")];

        let block_stream = BlockStream::new(
            file_walker,
            SchemaFormat::Json,
            &selection,
            Sorter::default(),
        );

        let expected = vec![
            (Cow::Owned(root_dir.join("0").join("0_0").join("0_0_0")), Value::from("0_0_0")),
            (Cow::Owned(root_dir.join("0").join("0_0").join("0_0_1").join("0_0_1_1")), Value::from("0_0_1_1")),
            (Cow::Owned(root_dir.join("0").join("0_0").join("0_0_2")), Value::from("0_0_2")),
            (Cow::Owned(root_dir.join("0").join("0_1").join("0_1_0")), Value::from("0_1_0")),
            (Cow::Owned(root_dir.join("0").join("0_1").join("0_1_1").join("0_1_1_1")), Value::from("0_1_1_1")),
            (Cow::Owned(root_dir.join("0").join("0_1").join("0_1_2")), Value::from("0_1_2")),
            (Cow::Owned(root_dir.join("0").join("0_2").join("0_2_0")), Value::from("0_2_0")),
            (Cow::Owned(root_dir.join("0").join("0_2").join("0_2_1").join("0_2_1_1")), Value::from("0_2_1_1")),
            (Cow::Owned(root_dir.join("0").join("0_2").join("0_2_2")), Value::from("0_2_2")),
        ];
        let produced = {
            ValueStream::new(target_key_path.clone(), block_stream)
                .into_iter()
                .map(|res| res.unwrap())
                .collect::<Vec<_>>()
        };

        assert_eq!(expected, produced);
    }

    #[test]
    fn meta_field_stream_none() {
        let temp_dir = TestUtil::create_meta_fanout_test_dir("meta_field_stream_none", 3, 3, TestUtil::flag_set_by_none);
        let root_dir = temp_dir.path();
        let selection = Selection::default();

        let origin_path = root_dir.join("0").join("0_1").join("0_1_2");
        let file_walker = FileWalker::Parent(ParentFileWalker::new(&origin_path));

        let target_key_path = vec![String::from("flag_key")];

        let block_stream = BlockStream::new(
            file_walker,
            SchemaFormat::Json,
            &selection,
            Sorter::default(),
        );

        let expected: Vec<(Cow<'_, _>, Value)> = vec![];
        let produced = {
            ValueStream::new(target_key_path.clone(), block_stream)
                .into_iter()
                .map(|res| res.unwrap())
                .collect::<Vec<_>>()
        };

        assert_eq!(expected, produced);

        let origin_path = root_dir.join("0");
        let file_walker = FileWalker::Child(ChildFileWalker::new(&origin_path));

        let target_key_path = vec![String::from("flag_key")];

        let block_stream = BlockStream::new(
            file_walker,
            SchemaFormat::Json,
            &selection,
            Sorter::default(),
        );

        let expected: Vec<(Cow<'_, _>, Value)> = vec![];
        let produced = {
            ValueStream::new(target_key_path.clone(), block_stream)
                .into_iter()
                .map(|res| res.unwrap())
                .collect::<Vec<_>>()
        };

        assert_eq!(expected, produced);
    }
}
