//! Iterators that yield meta blocks. This provides a layer of abstraction for later processes that
//! need a stream of meta blocks from various sources.

use std::borrow::Cow;
use std::path::Path;
use std::collections::VecDeque;

use config::selection::Selection;
use config::sort_order::SortOrder;
use config::meta_format::MetaFormat;
use metadata::types::MetaBlock;
use metadata::processor::MetaProcessor;
use metadata::processor::Error as ProcessorError;
use util::file_walkers::FileWalker;
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

pub enum MetaBlockProducer<'p, 's> {
    Fixed(FixedMetaBlockProducer<'p>),
    File(FileMetaBlockProducer<'p, 's>),
}

impl<'p, 's> Iterator for MetaBlockProducer<'p, 's> {
    type Item = Result<(Cow<'p, Path>, MetaBlock), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            &mut Self::Fixed(ref mut it) => it.next(),
            &mut Self::File(ref mut it) => it.next(),
        }
    }
}

impl<'p, 's> MetaBlockProducer<'p, 's> {
    pub fn delve(&mut self) -> Result<(), Error> {
        match self {
            &mut Self::Fixed(..) => Ok(()),
            &mut Self::File(ref mut producer) => producer.delve(),
        }
    }
}

/// A meta block producer that yields from a fixed sequence, used for testing.
pub struct FixedMetaBlockProducer<'p>(VecDeque<(Cow<'p, Path>, MetaBlock)>);

impl<'p> Iterator for FixedMetaBlockProducer<'p> {
    type Item = Result<(Cow<'p, Path>, MetaBlock), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop_front().map(Result::Ok)
    }
}

/// A meta block producer that yields from files on disk, powered by a file walker.
pub struct FileMetaBlockProducer<'p, 's> {
    file_walker: FileWalker<'p>,
    meta_format: MetaFormat,
    selection: &'s Selection,
    sort_order: SortOrder,
}

impl<'p, 's> Iterator for FileMetaBlockProducer<'p, 's> {
    type Item = Result<(Cow<'p, Path>, MetaBlock), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.file_walker.next() {
            Some(path_res) => {
                match path_res {
                    Ok(path) => {
                        Some(
                            MetaProcessor::process_item_file(
                                &path,
                                self.meta_format,
                                self.selection,
                                self.sort_order,
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

impl<'p, 's> FileMetaBlockProducer<'p, 's> {
    pub fn delve(&mut self) -> Result<(), Error> {
        self.file_walker.delve(&self.selection, self.sort_order).map_err(Error::FileWalker)
    }
}

#[cfg(test)]
mod tests {
    use super::FixedMetaBlockProducer;
    use super::FileMetaBlockProducer;

    use std::borrow::Cow;
    use std::path::Path;
    use std::collections::VecDeque;
    use test_util::TestUtil;

    use bigdecimal::BigDecimal;

    use metadata::types::MetaKey;
    use metadata::types::MetaVal;
    use config::selection::Selection;
    use config::sort_order::SortOrder;
    use config::meta_format::MetaFormat;
    use util::file_walkers::FileWalker;
    use util::file_walkers::ParentFileWalker;
    use util::file_walkers::ChildFileWalker;

    #[test]
    fn test_fixed_meta_block_producer() {
        let mb_a = btreemap![
            MetaKey::from("key_a") => MetaVal::Bul(true),
            MetaKey::from("key_b") => MetaVal::Dec(BigDecimal::from(3.1415)),
        ];
        let mb_b = btreemap![
            MetaKey::from("key_a") => MetaVal::Int(-1),
            MetaKey::from("key_b") => MetaVal::Nil,
        ];

        let mut vd = VecDeque::new();
        vd.push_back((Cow::Borrowed(Path::new("dummy_a")), mb_a.clone()));
        vd.push_back((Cow::Borrowed(Path::new("dummy_b")), mb_b.clone()));

        let mut producer = FixedMetaBlockProducer(vd);

        assert_eq!(
            producer.next().unwrap().unwrap(),
            (Cow::Borrowed(Path::new("dummy_a")), mb_a.clone()),
        );
        assert_eq!(
            producer.next().unwrap().unwrap(),
            (Cow::Borrowed(Path::new("dummy_b")), mb_b.clone()),
        );
        assert!(producer.next().is_none());
    }

    #[test]
    fn test_file_meta_block_producer() {
        let temp_dir = TestUtil::create_meta_fanout_test_dir("test_file_meta_block_producer");
        let root_dir = temp_dir.path();

        let test_path = root_dir.join("0").join("0_1").join("0_1_2");

        let mut producer = FileMetaBlockProducer {
            file_walker: FileWalker::Parent(ParentFileWalker::new(&test_path)),
            meta_format: MetaFormat::Json,
            selection: &Selection::default(),
            sort_order: SortOrder::Name,
        };

        assert_eq!(producer.next().unwrap().map(|(_, mb)| mb).unwrap().get(&MetaKey::from("target_file_name")), Some(&MetaVal::from("0_1_2")));
        assert_eq!(producer.next().unwrap().map(|(_, mb)| mb).unwrap().get(&MetaKey::from("target_file_name")), Some(&MetaVal::from("0_1")));
        assert_eq!(producer.next().unwrap().map(|(_, mb)| mb).unwrap().get(&MetaKey::from("target_file_name")), Some(&MetaVal::from("0")));
        assert_eq!(producer.next().unwrap().map(|(_, mb)| mb).unwrap().get(&MetaKey::from("target_file_name")), Some(&MetaVal::from("ROOT")));

        let test_path = root_dir.clone();

        let mut producer = FileMetaBlockProducer {
            file_walker: FileWalker::Child(ChildFileWalker::new(&test_path)),
            meta_format: MetaFormat::Json,
            selection: &Selection::default(),
            sort_order: SortOrder::Name,
        };

        assert_eq!(producer.next().unwrap().map(|(_, mb)| mb).unwrap().get(&MetaKey::from("target_file_name")), Some(&MetaVal::from("ROOT")));
        assert!(producer.next().is_none());

        producer.delve().unwrap();

        assert_eq!(producer.next().unwrap().map(|(_, mb)| mb).unwrap().get(&MetaKey::from("target_file_name")), Some(&MetaVal::from("0")));
        assert_eq!(producer.next().unwrap().map(|(_, mb)| mb).unwrap().get(&MetaKey::from("target_file_name")), Some(&MetaVal::from("1")));
        assert_eq!(producer.next().unwrap().map(|(_, mb)| mb).unwrap().get(&MetaKey::from("target_file_name")), Some(&MetaVal::from("2")));
        assert!(producer.next().is_none());

        producer.delve().unwrap();

        assert_eq!(producer.next().unwrap().map(|(_, mb)| mb).unwrap().get(&MetaKey::from("target_file_name")), Some(&MetaVal::from("2_0")));
        assert_eq!(producer.next().unwrap().map(|(_, mb)| mb).unwrap().get(&MetaKey::from("target_file_name")), Some(&MetaVal::from("2_1")));
        assert_eq!(producer.next().unwrap().map(|(_, mb)| mb).unwrap().get(&MetaKey::from("target_file_name")), Some(&MetaVal::from("2_2")));
        assert!(producer.next().is_none());
    }
}
