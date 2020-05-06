use std::borrow::Cow;
use std::path::Path;


use super::Error;

use crate::config::selection::Selection;
use crate::config::sorter::Sorter;
use crate::metadata::schema::SchemaFormat;
use crate::metadata::block::Block;
use crate::metadata::processor::Processor;
use crate::util::file_walker::FileWalker;

/// An iterator that yields metadata blocks from files on disk, powered by a file walker.
#[derive(Debug)]
pub struct BlockStream<'p> {
    file_walker: FileWalker<'p>,
    schema_format: &'p SchemaFormat,
    selection: &'p Selection,
    sorter: &'p Sorter,
}

impl<'p> BlockStream<'p> {
    pub fn new(
        file_walker: FileWalker<'p>,
        schema_format: &'p SchemaFormat,
        selection: &'p Selection,
        sorter: &'p Sorter,
    ) -> Self
    {
        Self {
            file_walker,
            schema_format,
            selection,
            sorter,
        }
    }

    pub fn delve(&mut self) -> Result<(), Error> {
        self.file_walker.delve(&self.selection, self.sorter).map_err(Error::FileWalker)
    }
}

impl<'p> Iterator for BlockStream<'p> {
    type Item = Result<(Cow<'p, Path>, Block), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.file_walker.next()? {
            Ok(path) => {
                Some(
                    Processor::process_item_file(
                        &path,
                        &self.schema_format,
                        self.selection,
                        &self.sorter,
                    )
                    .map(|mb| (path, mb))
                    .map_err(Error::Processor)
                )
            },
            Err(err) => Some(Err(Error::FileWalker(err))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_util::TestUtil;

    use crate::metadata::value::Value;
    use crate::util::file_walker::ParentFileWalker;
    use crate::util::file_walker::ChildFileWalker;

    #[test]
    fn file_meta_block_stream() {
        let temp_dir = TestUtil::create_meta_fanout_test_dir("file_meta_block_stream", 3, 3, TestUtil::flag_set_by_default);
        let root_dir = temp_dir.path();

        let selection = Selection::default();
        let sorter = Sorter::default();

        let test_path = root_dir.join("0").join("0_1").join("0_1_2");

        let mut stream = BlockStream::new(
            ParentFileWalker::new(&test_path).into(),
            &SchemaFormat::Json,
            &selection,
            &sorter,
        );

        assert_eq!(stream.next().unwrap().map(|(_, mb)| mb).unwrap().get("target_file_name"), Some(&Value::from("0_1_2")));
        assert_eq!(stream.next().unwrap().map(|(_, mb)| mb).unwrap().get("target_file_name"), Some(&Value::from("0_1")));
        assert_eq!(stream.next().unwrap().map(|(_, mb)| mb).unwrap().get("target_file_name"), Some(&Value::from("0")));
        assert_eq!(stream.next().unwrap().map(|(_, mb)| mb).unwrap().get("target_file_name"), Some(&Value::from("ROOT")));

        let test_path = root_dir.clone();

        let mut stream = BlockStream::new(
            ChildFileWalker::new(&test_path).into(),
            &SchemaFormat::Json,
            &selection,
            &sorter,
        );

        assert_eq!(stream.next().unwrap().map(|(_, mb)| mb).unwrap().get(&String::from("target_file_name")), Some(&Value::from("ROOT")));
        assert!(stream.next().is_none());

        stream.delve().unwrap();

        assert_eq!(stream.next().unwrap().map(|(_, mb)| mb).unwrap().get(&String::from("target_file_name")), Some(&Value::from("0")));
        assert_eq!(stream.next().unwrap().map(|(_, mb)| mb).unwrap().get(&String::from("target_file_name")), Some(&Value::from("1")));
        assert_eq!(stream.next().unwrap().map(|(_, mb)| mb).unwrap().get(&String::from("target_file_name")), Some(&Value::from("2")));
        assert!(stream.next().is_none());

        stream.delve().unwrap();

        assert_eq!(stream.next().unwrap().map(|(_, mb)| mb).unwrap().get(&String::from("target_file_name")), Some(&Value::from("2_0")));
        assert_eq!(stream.next().unwrap().map(|(_, mb)| mb).unwrap().get(&String::from("target_file_name")), Some(&Value::from("2_1")));
        assert_eq!(stream.next().unwrap().map(|(_, mb)| mb).unwrap().get(&String::from("target_file_name")), Some(&Value::from("2_2")));
        assert!(stream.next().is_none());
    }
}
