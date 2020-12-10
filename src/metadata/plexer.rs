//! Methods to assign blocks of metadata to their corresponding item file paths.

use std::borrow::Cow;
use std::io::{Error as IoError, Result as IoResult};
use std::iter::FusedIterator;
use std::path::Path;
use std::path::PathBuf;
use std::vec::IntoIter as VecIntoIter;

use thiserror::Error;

use crate::config::Sorter;
use crate::types::{Block, BlockMap};
use crate::types::block_seq::IntoIter as BlockSeqIntoIter;
use crate::metadata::schema::Schema;

#[derive(Debug, Error)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] IoError),
    #[error("item path was unused: {}", .0.display())]
    UnusedItemPath(PathBuf),
    #[error("meta block was unused")]
    UnusedBlock(Block),
    #[error(r#"meta block was unused, with tag "{1}""#)]
    UnusedTaggedBlock(Block, String),
    #[error("item path does not have a file name: {}", .0.display())]
    NamelessItemPath(PathBuf),
}

type PlexInItem<'a> = IoResult<Cow<'a, Path>>;
type PlexOutItem<'a> = Result<(Cow<'a, Path>, Block), Error>;

fn pair_up<'a>(
    opt_block: Option<Block>,
    opt_path: Option<Cow<'a, Path>>,
) -> Option<PlexOutItem<'a>> {
    match (opt_block, opt_path) {
        // Both iterators are exhausted, so this one is as well.
        (None, None) => None,

        // Both iterators produced a result, emit a successful plex result.
        (Some(block), Some(path)) => Some(Ok((path, block))),

        // Got a file path with no meta block, report an error.
        (None, Some(path)) => Some(Err(Error::UnusedItemPath(path.into_owned()))),

        // Got a meta block with no file path, report an error.
        (Some(block), None) => Some(Err(Error::UnusedBlock(block))),
    }
}

pub struct PlexOne<'a, I>(Option<Block>, I)
where
    I: Iterator<Item = PlexInItem<'a>>;

impl<'a, I> Iterator for PlexOne<'a, I>
where
    I: Iterator<Item = PlexInItem<'a>>,
{
    type Item = PlexOutItem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let res = self.1.next().transpose();

        match res {
            Err(err) => Some(Err(Error::Io(err))),
            Ok(opt_path) => pair_up(self.0.take(), opt_path)
        }
    }
}

pub struct PlexSeq<'a> {
    block_iter: BlockSeqIntoIter,
    err_iter: VecIntoIter<IoError>,
    path_iter: VecIntoIter<Cow<'a, Path>>
}

impl<'a> Iterator for PlexSeq<'a> {
    type Item = PlexOutItem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(err) = self.err_iter.next() {
            Some(Err(Error::Io(err)))
        } else {
            pair_up(self.block_iter.next(), self.path_iter.next())
        }
    }
}

pub struct PlexMap<'a, I>(BlockMap, I)
where
    I: Iterator<Item = PlexInItem<'a>>;

impl<'a, I> Iterator for PlexMap<'a, I>
where
    I: Iterator<Item = PlexInItem<'a>>,
{
    type Item = PlexOutItem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.1.next() {
            Some(Err(err)) => Some(Err(Error::Io(err))),
            Some(Ok(path)) => {
                // Try and obtain a file name from the path, and convert into a
                // string for lookup. If this fails, return an error for this
                // iteration and then skip the string.
                match path.file_name().and_then(|os| os.to_str()) {
                    None => Some(Err(Error::NamelessItemPath(path.into()))),
                    Some(name_tag) => {
                        // See if the tag is in the meta block mapping.
                        match self.0.remove(name_tag) {
                            // No meta block in the mapping had a matching tag, report an error.
                            None => Some(Err(Error::UnusedItemPath(path.into()))),

                            // Found a matching meta block, emit a successful plex result.
                            Some(block) => Some(Ok((path, block))),
                        }
                    }
                }
            }
            None => {
                // No more file paths, see if there are any more meta blocks.
                match self.0.pop() {
                    // Found an orphaned meta block, report an error.
                    Some((name_tag, block)) => Some(Err(Error::UnusedTaggedBlock(block, name_tag))),

                    // No more meta blocks were found, this iterator is now exhausted.
                    None => None,
                }
            }
        }
    }
}

pub enum Plexer<'a, I>
where
    I: Iterator<Item = PlexInItem<'a>>,
{
    One(PlexOne<'a, I>),
    Seq(PlexSeq<'a>),
    Map(PlexMap<'a, I>),
}

impl<'a, I> Iterator for Plexer<'a, I>
where
    I: Iterator<Item = PlexInItem<'a>>,
{
    type Item = PlexOutItem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::One(it) => it.next(),
            Self::Seq(it) => it.next(),
            Self::Map(it) => it.next(),
        }
    }
}

impl<'a, I> FusedIterator for Plexer<'a, I> where I: Iterator<Item = PlexInItem<'a>> {}

impl<'a, I> Plexer<'a, I>
where
    I: Iterator<Item = PlexInItem<'a>>,
{
    /// Creates a new `Plexer`.
    pub fn new<II>(schema: Schema, file_path_iter: II, sorter: &Sorter) -> Self
    where
        II: IntoIterator<IntoIter = I, Item = I::Item>,
    {
        let file_path_iter = file_path_iter.into_iter();

        match schema {
            Schema::One(mb) => Self::One(PlexOne(Some(mb), file_path_iter)),
            Schema::Seq(mb_seq) => {
                // Need to pre-collect, in order to sort.
                // Since the entire path iterator needs to be read right now,
                // just pre-partion the path results into `Ok`/`Err`s.
                let mut errs = Vec::new();
                let mut paths = Vec::new();

                for res in file_path_iter {
                    match res {
                        Err(err) => { errs.push(err); },
                        Ok(path) => { paths.push(path); }
                    }
                }

                sorter.sort_paths(&mut paths);

                let plex_seq = PlexSeq {
                    block_iter: mb_seq.into_iter(),
                    err_iter: errs.into_iter(),
                    path_iter: paths.into_iter(),
                };

                Self::Seq(plex_seq)
            }
            Schema::Map(mb_map) => Self::Map(PlexMap(mb_map, file_path_iter)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // use std::collections::HashSet;

    use indexmap::indexmap;
    use maplit::btreemap;
    use str_macro::str;

    use crate::types::{Block, BlockSeq, BlockMap};

    use crate::test_util::TestUtil as TU;

    // Helper macros.
    macro_rules! assert_ok {
        ( $plex:expr, $path:expr, $block:expr ) => {
            match $plex.next() {
                Some(Ok((ref p, ref b))) => {
                    assert_eq!(p, &$path);
                    assert_eq!(b, &$block);
                }
                Some(Err(e)) => panic!("unexpected error: {}", e),
                None => panic!("unexpected none"),
            };
        };
    }
    macro_rules! assert_none {
        ( $plex:expr ) => {
            match $plex.next() {
                Some(Ok((ref p, ref b))) => panic!("unexpected ok: ({}, {:?})", p.display(), b),
                Some(Err(e)) => panic!("unexpected error: {}", e),
                None => {}
            };
        };
    }
    macro_rules! assert_extra_path {
        ( $plex:expr, $path:expr ) => {
            match $plex.next() {
                Some(Err(Error::UnusedItemPath(ref p))) => {
                    assert_eq!(p, &$path);
                }
                Some(Err(e)) => panic!("unexpected error: {}", e),
                Some(Ok((ref p, ref b))) => panic!("unexpected ok: ({}, {:?})", p.display(), b),
                None => panic!("unexpected none"),
            };
        };
    }
    macro_rules! assert_extra_block {
        ( $plex:expr, $block:expr ) => {
            match $plex.next() {
                Some(Err(Error::UnusedBlock(ref b))) => {
                    assert_eq!(b, &$block);
                }
                Some(Err(e)) => panic!("unexpected error: {}", e),
                Some(Ok((ref p, ref b))) => panic!("unexpected ok: ({}, {:?})", p.display(), b),
                None => panic!("unexpected none"),
            };
        };
    }
    macro_rules! assert_extra_tagged_block {
        ( $plex:expr, $block:expr, $tag:expr ) => {
            match $plex.next() {
                Some(Err(Error::UnusedTaggedBlock(ref b, ref t))) => {
                    assert_eq!(b, &$block);
                    assert_eq!(t, &$tag);
                }
                Some(Err(e)) => panic!("unexpected error: {}", e),
                Some(Ok((ref p, ref b))) => panic!("unexpected ok: ({}, {:?})", p.display(), b),
                None => panic!("unexpected none"),
            };
        };
    }
    macro_rules! assert_io_error {
        ( $plex:expr ) => {
            match $plex.next() {
                Some(Err(Error::Io(..))) => {},
                Some(Err(e)) => panic!("unexpected error: {}", e),
                Some(Ok((ref p, ref b))) => panic!("unexpected ok: ({}, {:?})", p.display(), b),
                None => panic!("unexpected none"),
            };
        };
    }
    macro_rules! assert_nameless_path {
        ( $plex:expr, $path:expr ) => {
            match $plex.next() {
                Some(Err(Error::NamelessItemPath(ref p))) => {
                    assert_eq!(p, &$path);
                },
                Some(Err(e)) => panic!("unexpected error: {}", e),
                Some(Ok((ref p, ref b))) => panic!("unexpected ok: ({}, {:?})", p.display(), b),
                None => panic!("unexpected none"),
            };
        };
    }

    // Helper method.
    fn okc<'a>(path: &'a Path) -> PlexInItem<'a> {
        Ok(Cow::Borrowed(path))
    }

    #[test]
    fn plex() {
        let block_a = Block(btreemap![str!("key_a") => TU::s("val_a")]);
        let block_b = Block(btreemap![str!("key_b") => TU::s("val_b")]);
        let block_c = Block(btreemap![str!("key_c") => TU::s("val_c")]);

        let name_a = "name_a";
        let name_b = "name_b";
        let name_c = "name_c";

        let path_a = Path::new(name_a);
        let path_b = Path::new(name_b);
        let path_c = Path::new(name_c);
        let path_x = Path::new("xx_missing_xx");

        let sorter = Sorter::default();

        let schema_one = Schema::One(block_a.clone());
        let schema_seq = Schema::Seq(BlockSeq(vec![block_a.clone(), block_b.clone(), block_c.clone()]));
        let schema_map = Schema::Map(BlockMap(indexmap![
            str!(name_c) => block_c.clone(),
            str!(name_a) => block_a.clone(),
            str!(name_b) => block_b.clone(),
        ]));

        // Testing `Schema::One`.
        let schema = schema_one;

        // Normal case.
        let mut plexer = Plexer::new(schema.clone(), vec![okc(&path_a)], &sorter);
        assert_ok!(plexer, path_a, block_a);
        assert_none!(plexer);

        // Too many paths.
        let mut plexer = Plexer::new(
            schema.clone(),
            vec![okc(&path_a), okc(&path_x)],
            &sorter,
        );
        assert_ok!(plexer, path_a, block_a);
        assert_extra_path!(plexer, path_x);
        assert_none!(plexer);

        // Not enough paths.
        let mut plexer = Plexer::new(schema.clone(), vec![], &sorter);
        assert_extra_block!(plexer, block_a);
        assert_none!(plexer);

        // IO error.
        let mut plexer = Plexer::new(
            schema.clone(),
            vec![
                Err(IoError::new(std::io::ErrorKind::Other, "sample")),
                okc(&path_a),
            ],
            &sorter,
        );
        assert_io_error!(plexer);
        assert_ok!(plexer, path_a, block_a);
        assert_none!(plexer);

        // Testing `Schema::Seq`.
        let schema = schema_seq;

        // Normal case.
        let mut plexer = Plexer::new(
            schema.clone(),
            vec![okc(&path_a), okc(&path_b), okc(&path_c)],
            &sorter,
        );
        assert_ok!(plexer, path_a, block_a);
        assert_ok!(plexer, path_b, block_b);
        assert_ok!(plexer, path_c, block_c);
        assert_none!(plexer);

        // Too many paths.
        let mut plexer = Plexer::new(
            schema.clone(),
            vec![okc(&path_a), okc(&path_b), okc(&path_c), okc(&path_x)],
            &sorter,
        );
        assert_ok!(plexer, path_a, block_a);
        assert_ok!(plexer, path_b, block_b);
        assert_ok!(plexer, path_c, block_c);
        assert_extra_path!(plexer, path_x);
        assert_none!(plexer);

        // Not enough paths.
        let mut plexer = Plexer::new(
            schema.clone(),
            vec![okc(&path_a), okc(&path_b)],
            &sorter,
        );
        assert_ok!(plexer, path_a, block_a);
        assert_ok!(plexer, path_b, block_b);
        assert_extra_block!(plexer, block_c);
        assert_none!(plexer);

        // IO error.
        let mut plexer = Plexer::new(
            schema.clone(),
            vec![
                okc(&path_a),
                okc(&path_b),
                okc(&path_c),
                Err(IoError::new(std::io::ErrorKind::Other, "sample")),
            ],
            &sorter,
        );
        assert_io_error!(plexer);
        assert_ok!(plexer, path_a, block_a);
        assert_ok!(plexer, path_b, block_b);
        assert_ok!(plexer, path_c, block_c);
        assert_none!(plexer);

        // Testing `Schema::Map`.
        let schema = schema_map;

        // Normal case.
        let mut plexer = Plexer::new(
            schema.clone(),
            vec![okc(&path_a), okc(&path_b), okc(&path_c)],
            &sorter,
        );
        assert_ok!(plexer, path_a, block_a);
        assert_ok!(plexer, path_b, block_b);
        assert_ok!(plexer, path_c, block_c);
        assert_none!(plexer);

        // Too many paths.
        let mut plexer = Plexer::new(
            schema.clone(),
            vec![okc(&path_x), okc(&path_a), okc(&path_b), okc(&path_c)],
            &sorter,
        );
        assert_extra_path!(plexer, path_x);
        assert_ok!(plexer, path_a, block_a);
        assert_ok!(plexer, path_b, block_b);
        assert_ok!(plexer, path_c, block_c);
        assert_none!(plexer);

        // Not enough paths.
        let mut plexer = Plexer::new(
            schema.clone(),
            vec![okc(&path_a), okc(&path_b)],
            &sorter,
        );
        assert_ok!(plexer, path_a, block_a);
        assert_ok!(plexer, path_b, block_b);
        assert_extra_tagged_block!(plexer, block_c, name_c);
        assert_none!(plexer);

        // IO error.
        let mut plexer = Plexer::new(
            schema.clone(),
            vec![
                okc(&path_a),
                okc(&path_b),
                Err(IoError::new(std::io::ErrorKind::Other, "sample")),
                okc(&path_c),
            ],
            &sorter,
        );
        assert_ok!(plexer, path_a, block_a);
        assert_ok!(plexer, path_b, block_b);
        assert_io_error!(plexer);
        assert_ok!(plexer, path_c, block_c);
        assert_none!(plexer);

        // Nameless path.
        let nameless = Path::new("/");
        let mut plexer = Plexer::new(
            schema.clone(),
            vec![
                okc(&path_a),
                okc(&path_b),
                okc(&nameless),
                okc(&path_c),
            ],
            &sorter,
        );
        assert_ok!(plexer, path_a, block_a);
        assert_ok!(plexer, path_b, block_b);
        assert_nameless_path!(plexer, nameless);
        assert_ok!(plexer, path_c, block_c);
        assert_none!(plexer);
    }
}
