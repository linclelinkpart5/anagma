//! Iterators that yield meta blocks. This provides a layer of abstraction for later processes that
//! need a stream of meta blocks from various sources.

use std::borrow::Cow;
use std::path::Path;
use std::collections::VecDeque;

use crate::config::selection::Selection;
use crate::config::sorter::Sorter;
use crate::config::meta_format::MetaFormat;
use crate::metadata::block::Block;
use crate::metadata::processor::Processor;
use crate::metadata::processor::Error as ProcessorError;
use crate::util::file_walkers::FileWalker;
use crate::util::file_walkers::Error as FileWalkerError;

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

/// Generic meta block stream, that can be fed in a variety of ways.
#[derive(Debug)]
pub enum BlockStream<'p> {
    Fixed(FixedBlockStream<'p>),
    File(FileBlockStream<'p>),
}

impl<'p> Iterator for BlockStream<'p> {
    type Item = Result<(Cow<'p, Path>, Block), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            &mut Self::Fixed(ref mut it) => it.next(),
            &mut Self::File(ref mut it) => it.next(),
        }
    }
}

impl<'p> BlockStream<'p> {
    pub fn delve(&mut self) -> Result<(), Error> {
        match self {
            &mut Self::Fixed(..) => Ok(()),
            &mut Self::File(ref mut stream) => stream.delve(),
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

/// A meta block stream that yields from a fixed sequence, used for testing.
#[derive(Debug)]
pub struct FixedBlockStream<'p>(VecDeque<(Cow<'p, Path>, Block)>);

impl<'p> FixedBlockStream<'p> {
    pub fn new<II>(items: II) -> Self
    where
        II: IntoIterator<Item = (Cow<'p, Path>, Block)>,
    {
        FixedBlockStream(items.into_iter().collect())
    }
}

impl<'p> Iterator for FixedBlockStream<'p> {
    type Item = Result<(Cow<'p, Path>, Block), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop_front().map(Result::Ok)
    }
}

/// A meta block stream that yields from files on disk, powered by a file walker.
#[derive(Debug)]
pub struct FileBlockStream<'p> {
    file_walker: FileWalker<'p>,
    meta_format: MetaFormat,
    selection: &'p Selection,
    sorter: Sorter,
}

impl<'p> FileBlockStream<'p> {
    pub fn new<FW>(
        file_walker: FW,
        meta_format: MetaFormat,
        selection: &'p Selection,
        sorter: Sorter,
    ) -> Self
    where
        FW: Into<FileWalker<'p>>,
    {
        FileBlockStream {
            file_walker: file_walker.into(),
            meta_format,
            selection,
            sorter,
        }
    }
}

impl<'p> Iterator for FileBlockStream<'p> {
    type Item = Result<(Cow<'p, Path>, Block), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.file_walker.next() {
            Some(path_res) => {
                match path_res {
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
            },
            None => None,
        }
    }
}

impl<'p> FileBlockStream<'p> {
    pub fn delve(&mut self) -> Result<(), Error> {
        self.file_walker.delve(&self.selection, self.sorter).map_err(Error::FileWalker)
    }
}

#[cfg(test)]
mod tests {
    use super::FixedBlockStream;
    use super::FileBlockStream;

    use std::borrow::Cow;
    use std::path::Path;
    use std::collections::VecDeque;
    use crate::test_util::TestUtil;

    use crate::metadata::value::Value;
    use crate::config::selection::Selection;
    use crate::config::sorter::Sorter;
    use crate::config::meta_format::MetaFormat;
    use crate::util::file_walkers::FileWalker;
    use crate::util::file_walkers::ParentFileWalker;
    use crate::util::file_walkers::ChildFileWalker;

    #[test]
    fn test_fixed_meta_block_stream() {
        let mb_a = btreemap![
            String::from("key_a") => Value::Boolean(true),
            String::from("key_b") => Value::Decimal(dec!(3.1415)),
        ];
        let mb_b = btreemap![
            String::from("key_a") => Value::Integer(-1),
            String::from("key_b") => Value::Null,
        ];

        let mut vd = VecDeque::new();
        vd.push_back((Cow::Borrowed(Path::new("dummy_a")), mb_a.clone()));
        vd.push_back((Cow::Borrowed(Path::new("dummy_b")), mb_b.clone()));

        let mut stream = FixedBlockStream(vd);

        assert_eq!(
            stream.next().unwrap().unwrap(),
            (Cow::Borrowed(Path::new("dummy_a")), mb_a.clone()),
        );
        assert_eq!(
            stream.next().unwrap().unwrap(),
            (Cow::Borrowed(Path::new("dummy_b")), mb_b.clone()),
        );
        assert!(stream.next().is_none());
    }

    #[test]
    fn test_file_meta_block_stream() {
        let temp_dir = TestUtil::create_meta_fanout_test_dir("test_file_meta_block_stream", 3, 3, TestUtil::flag_set_by_default);
        let root_dir = temp_dir.path();

        let test_path = root_dir.join("0").join("0_1").join("0_1_2");

        let mut stream = FileBlockStream {
            file_walker: FileWalker::Parent(ParentFileWalker::new(&test_path)),
            meta_format: MetaFormat::Json,
            selection: &Selection::default(),
            sorter: Sorter::default(),
        };

        assert_eq!(stream.next().unwrap().map(|(_, mb)| mb).unwrap().get(&String::from("target_file_name")), Some(&Value::from("0_1_2")));
        assert_eq!(stream.next().unwrap().map(|(_, mb)| mb).unwrap().get(&String::from("target_file_name")), Some(&Value::from("0_1")));
        assert_eq!(stream.next().unwrap().map(|(_, mb)| mb).unwrap().get(&String::from("target_file_name")), Some(&Value::from("0")));
        assert_eq!(stream.next().unwrap().map(|(_, mb)| mb).unwrap().get(&String::from("target_file_name")), Some(&Value::from("ROOT")));

        let test_path = root_dir.clone();

        let mut stream = FileBlockStream {
            file_walker: FileWalker::Child(ChildFileWalker::new(test_path)),
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
