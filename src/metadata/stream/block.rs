use std::borrow::Cow;
use std::path::Path;

use super::Error;

use crate::config::selection::Selection;
use crate::config::sorter::Sorter;
use crate::config::meta_format::MetaFormat;
use crate::metadata::block::Block;
use crate::metadata::processor::Processor;
use crate::util::file_walker::FileWalker;

/// An iterator that yields metadata blocks. These provide a layer of abstraction
/// for later processes that need a stream of meta blocks from various sources.
#[derive(Debug)]
pub enum BlockStream<'p> {
    Fixed(FixedBlockStream<'p>),
    File(FileBlockStream<'p>),
}

impl<'p> Iterator for BlockStream<'p> {
    type Item = Result<(Cow<'p, Path>, Block), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Fixed(ref mut it) => it.next(),
            Self::File(ref mut it) => it.next(),
        }
    }
}

impl<'p> BlockStream<'p> {
    pub fn delve(&mut self) -> Result<(), Error> {
        match self {
            Self::Fixed(..) => Ok(()),
            Self::File(ref mut stream) => stream.delve(),
        }
    }
}

impl<'p> From<FixedBlockStream<'p>> for BlockStream<'p> {
    fn from(other: FixedBlockStream<'p>) -> Self {
        Self::Fixed(other)
    }
}

impl<'p> From<FileBlockStream<'p>> for BlockStream<'p> {
    fn from(other: FileBlockStream<'p>) -> Self {
        Self::File(other)
    }
}

/// A block stream that yields from a fixed sequence, used for testing.
#[derive(Debug)]
pub struct FixedBlockStream<'p>(std::vec::IntoIter<(Cow<'p, Path>, Block)>);

impl<'p> FixedBlockStream<'p> {
    pub fn new(items: Vec<(Cow<'p, Path>, Block)>) -> Self {
        FixedBlockStream(items.into_iter())
    }
}

impl<'p> Iterator for FixedBlockStream<'p> {
    type Item = Result<(Cow<'p, Path>, Block), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(Result::Ok)
    }
}

/// A block stream that yields from files on disk, powered by a file walker.
#[derive(Debug)]
pub struct FileBlockStream<'p> {
    file_walker: FileWalker<'p>,
    meta_format: MetaFormat,
    selection: &'p Selection,
    sorter: Sorter,
}

impl<'p> FileBlockStream<'p> {
    pub fn new(
        file_walker: FileWalker<'p>,
        meta_format: MetaFormat,
        selection: &'p Selection,
        sorter: Sorter,
    ) -> Self
    {
        Self {
            file_walker,
            meta_format,
            selection,
            sorter,
        }
    }

    pub fn delve(&mut self) -> Result<(), Error> {
        self.file_walker.delve(&self.selection, self.sorter).map_err(Error::FileWalker)
    }
}

impl<'p> Iterator for FileBlockStream<'p> {
    type Item = Result<(Cow<'p, Path>, Block), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.file_walker.next()? {
            Ok(path) => {
                Some(
                    Processor::process_item_file(
                        &path,
                        self.meta_format,
                        self.selection,
                        self.sorter,
                    )
                    .map(|mb| (path, mb))
                    .map_err(Error::Processor)
                )
            },
            Err(err) => Some(Err(Error::FileWalker(err))),
        }
    }
}

#[cfg_attr(test, faux::create)]
pub struct NewBlockStream<'p> {
    file_walker: FileWalker<'p>,
    meta_format: MetaFormat,
    selection: &'p Selection,
    sorter: Sorter,
}

#[cfg_attr(test, faux::methods)]
impl<'p> NewBlockStream<'p> {
    pub fn new(
        file_walker: FileWalker<'p>,
        meta_format: MetaFormat,
        selection: &'p Selection,
        sorter: Sorter,
    ) -> Self
    {
        Self {
            file_walker,
            meta_format,
            selection,
            sorter,
        }
    }

    pub fn delve(&mut self) -> Result<(), Error> {
        self.file_walker.delve(&self.selection, self.sorter).map_err(Error::FileWalker)
    }
}

#[cfg_attr(test, faux::methods)]
impl<'p> Iterator for NewBlockStream<'p> {
    type Item = Result<(Cow<'p, Path>, Block), Error>;

    fn next(&mut self) -> Option<<NewBlockStream<'p> as Iterator>::Item> {
        match self.file_walker.next()? {
            Ok(path) => {
                Some(
                    Processor::process_item_file(
                        &path,
                        self.meta_format,
                        self.selection,
                        self.sorter,
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

    use maplit::btreemap;
    use rust_decimal_macros::dec;

    use crate::test_util::TestUtil;

    use crate::metadata::value::Value;
    use crate::util::file_walker::ParentFileWalker;
    use crate::util::file_walker::ChildFileWalker;

    #[test]
    fn mock_block_stream() {
        let mb_a = btreemap![
            String::from("key_a") => Value::Boolean(true),
            String::from("key_b") => Value::Decimal(dec!(3.1415)),
        ];
        let mb_b = btreemap![
            String::from("key_a") => Value::Integer(-1),
            String::from("key_b") => Value::Null,
        ];

        let stream = vec![
            (Cow::Borrowed(Path::new("dummy_a")), mb_a.clone()),
            (Cow::Borrowed(Path::new("dummy_b")), mb_b.clone()),
        ];
        let mut stream_iter = stream.into_iter().map(Result::Ok);

        let mut mock_block_stream = NewBlockStream::faux();

        faux::when!(mock_block_stream.next).safe_then(move |_| stream_iter.next());

        assert_eq!(
            mock_block_stream.next().unwrap().unwrap(),
            (Cow::Borrowed(Path::new("dummy_a")), mb_a),
        );
        assert_eq!(
            mock_block_stream.next().unwrap().unwrap(),
            (Cow::Borrowed(Path::new("dummy_b")), mb_b),
        );
        assert!(mock_block_stream.next().is_none());
    }

    #[test]
    fn fixed_meta_block_stream() {
        let mb_a = btreemap![
            String::from("key_a") => Value::Boolean(true),
            String::from("key_b") => Value::Decimal(dec!(3.1415)),
        ];
        let mb_b = btreemap![
            String::from("key_a") => Value::Integer(-1),
            String::from("key_b") => Value::Null,
        ];

        let mut stream = FixedBlockStream::new(vec![
            (Cow::Borrowed(Path::new("dummy_a")), mb_a.clone()),
            (Cow::Borrowed(Path::new("dummy_b")), mb_b.clone()),
        ]);

        assert_eq!(
            stream.next().unwrap().unwrap(),
            (Cow::Borrowed(Path::new("dummy_a")), mb_a),
        );
        assert_eq!(
            stream.next().unwrap().unwrap(),
            (Cow::Borrowed(Path::new("dummy_b")), mb_b),
        );
        assert!(stream.next().is_none());
    }

    #[test]
    fn file_meta_block_stream() {
        let temp_dir = TestUtil::create_meta_fanout_test_dir("file_meta_block_stream", 3, 3, TestUtil::flag_set_by_default);
        let root_dir = temp_dir.path();

        let test_path = root_dir.join("0").join("0_1").join("0_1_2");

        let mut stream = FileBlockStream {
            file_walker: ParentFileWalker::new(&test_path).into(),
            meta_format: MetaFormat::Json,
            selection: &Selection::default(),
            sorter: Sorter::default(),
        };

        assert_eq!(stream.next().unwrap().map(|(_, mb)| mb).unwrap().get("target_file_name"), Some(&Value::from("0_1_2")));
        assert_eq!(stream.next().unwrap().map(|(_, mb)| mb).unwrap().get("target_file_name"), Some(&Value::from("0_1")));
        assert_eq!(stream.next().unwrap().map(|(_, mb)| mb).unwrap().get("target_file_name"), Some(&Value::from("0")));
        assert_eq!(stream.next().unwrap().map(|(_, mb)| mb).unwrap().get("target_file_name"), Some(&Value::from("ROOT")));

        let test_path = root_dir.clone();

        let mut stream = FileBlockStream {
            file_walker: ChildFileWalker::new(&test_path).into(),
            meta_format: MetaFormat::Json,
            selection: &Selection::default(),
            sorter: Sorter::default(),
        };

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
