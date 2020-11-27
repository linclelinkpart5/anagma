//! Methods to assign blocks of metadata to their corresponding item file paths.

use std::borrow::Cow;
use std::io::{Error as IoError, Result as IoResult};
use std::iter::FusedIterator;
use std::path::Path;
use std::path::PathBuf;
use std::vec::IntoIter as VecIntoIter;

use thiserror::Error;

use crate::config::sorter::Sorter;
use crate::metadata::block::Block;
use crate::metadata::block::BlockMapping;
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
    opt_path_item: Option<PlexInItem<'a>>,
) -> Option<PlexOutItem<'a>> {
    match (opt_block, opt_path_item) {
        // If the path iterator produces an error, return it.
        (_, Some(Err(err))) => Some(Err(Error::Io(err))),

        // Both iterators are exhausted, so this one is as well.
        (None, None) => None,

        // Both iterators produced a result, emit a successful plex result.
        (Some(block), Some(Ok(path))) => Some(Ok((path, block))),

        // Got a file path with no meta block, report an error.
        (None, Some(Ok(path))) => Some(Err(Error::UnusedItemPath(path.into_owned()))),

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
        pair_up(self.0.take(), self.1.next())
    }
}

pub struct PlexSeq<'a>(VecIntoIter<Block>, VecIntoIter<PlexInItem<'a>>);

impl<'a> Iterator for PlexSeq<'a> {
    type Item = PlexOutItem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        pair_up(self.0.next(), self.1.next())
    }
}

pub struct PlexMap<'a, I>(BlockMapping, I)
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
                // TODO: Validate file name.
                match path.file_name().and_then(|os| os.to_str()) {
                    None => Some(Err(Error::NamelessItemPath(path.into()))),
                    Some(name_tag) => {
                        // See if the tag is in the meta block mapping.
                        match self.0.swap_remove(name_tag) {
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
                let mut file_paths = file_path_iter.collect::<Vec<_>>();
                sorter.sort_path_results(&mut file_paths);
                Self::Seq(PlexSeq(mb_seq.into_iter(), file_paths.into_iter()))
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

    // Helper method.
    fn okc<'a>(path: &'a Path) -> PlexInItem<'a> {
        Ok(Cow::Borrowed(path))
    }

    #[test]
    fn plex() {
        let block_a = btreemap![str!("key_a") => TU::s("val_a")];
        let block_b = btreemap![str!("key_b") => TU::s("val_b")];
        let block_c = btreemap![str!("key_c") => TU::s("val_c")];

        let name_a = "name_a";
        let name_b = "name_b";
        let name_c = "name_c";

        let path_a = Path::new(name_a);
        let path_b = Path::new(name_b);
        let path_c = Path::new(name_c);
        let path_x = Path::new("xx_missing_xx");

        let sorter = Sorter::default();

        let schema_one = Schema::One(block_a.clone());
        let schema_seq = Schema::Seq(vec![block_a.clone(), block_b.clone(), block_c.clone()]);
        let schema_map = Schema::Map(indexmap![
            str!(name_c) => block_c.clone(),
            str!(name_a) => block_a.clone(),
            str!(name_b) => block_b.clone(),
        ]);

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
        assert_extra_path!(plexer, path_a);
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
        assert_ok!(plexer, path_a, block_b);
        assert_ok!(plexer, path_b, block_c);
        assert_extra_path!(plexer, path_c);
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
    }

    #[test]
    fn plex_old() {
        // let block_a = btreemap![
        //     str!("key_1a") => TU::s("val_1a"),
        //     str!("key_1b") => TU::s("val_1b"),
        //     str!("key_1c") => TU::s("val_1c"),
        // ];
        // let block_b = btreemap![
        //     str!("key_2a") => TU::s("val_2a"),
        //     str!("key_2b") => TU::s("val_2b"),
        //     str!("key_2c") => TU::s("val_2c"),
        // ];
        // let block_c = btreemap![
        //     str!("key_3a") => TU::s("val_3a"),
        //     str!("key_3b") => TU::s("val_3b"),
        //     str!("key_3c") => TU::s("val_3c"),
        // ];

        // let block_seq = vec![block_a.clone(), block_b.clone(), block_c.clone()];

        // let block_map = indexmap![
        //     str!("item_c") => block_c.clone(),
        //     str!("item_a") => block_a.clone(),
        //     str!("item_b") => block_b.clone(),
        // ];

        // let schema_a = Schema::One(block_a.clone());
        // let schema_b = Schema::Seq(vec![block_a.clone(), block_b.clone(), block_c.clone()]);
        // let schema_c = Schema::Map(indexmap![
        //     str!("item_c") => block_c.clone(),
        //     str!("item_a") => block_a.clone(),
        //     str!("item_b") => block_b.clone(),
        // ]);

        // let path_a = Cow::Borrowed(Path::new("item_a"));
        // let path_b = Cow::Borrowed(Path::new("item_b"));

        // let sorter = Sorter::default();

        // // Single schemas.
        // let schema = Schema::One(block_a.clone());
        // let res_paths = vec![Ok(path_a)];
        // let mut plexer = Plexer::new(schema, res_paths, &sorter);
        // assert!(matches!(plexer.next(), Some(Ok((path_a, _)))));

        //     // Test single and sequence schemas.
        //     let inputs_and_expected = vec![
        //         (
        //             (schema_a.clone(), vec![path_a]),
        //             vec![
        //                 Ok((path_a, block_a.clone())),
        //             ],
        //         ),
        //         (
        //             (schema_a.clone(), vec![path_a, path_b]),
        //             vec![
        //                 Ok((path_a, block_a.clone())),
        //                 Err(Error::UnusedItemPath(path_b.clone().into_owned())),
        //             ],
        //         ),
        //         (
        //             (schema_a.clone(), vec![]),
        //             vec![
        //                 Err(Error::UnusedBlock(block_a.clone())),
        //             ],
        //         ),
        //         (
        //             (schema_b.clone(), vec![path_a, path_b, Cow::Owned(PathBuf::from("item_c"))]),
        //             vec![
        //                 Ok((path_a, block_a.clone())),
        //                 Ok((path_b, block_b.clone())),
        //                 Ok((Cow::Owned(PathBuf::from("item_c")), block_c.clone())),
        //             ],
        //         ),
        //         (
        //             (schema_b.clone(), vec![path_a, path_b, Cow::Owned(PathBuf::from("item_c")), Cow::Owned(PathBuf::from("item_d"))]),
        //             vec![
        //                 Ok((path_a, block_a.clone())),
        //                 Ok((path_b, block_b.clone())),
        //                 Ok((Cow::Owned(PathBuf::from("item_c")), block_c.clone())),
        //                 Err(Error::UnusedItemPath(PathBuf::from("item_d"))),
        //             ],
        //         ),
        //         (
        //             (schema_b.clone(), vec![path_a]),
        //             vec![
        //                 Ok((path_a, block_a.clone())),
        //                 Err(Error::UnusedBlock(block_b.clone())),
        //                 Err(Error::UnusedBlock(block_c.clone())),
        //             ],
        //         ),
        //     ];

        //     for (input, expected) in inputs_and_expected {
        //         let (meta_schema, item_paths) = input;
        //         let produced = Plexer::new(meta_schema, item_paths.into_iter().map(Result::Ok)).collect::<Vec<_>>();
        //         assert_eq!(expected, produced);
        //     }

        //     // Test mapping schemas.
        //     let inputs_and_expected = vec![
        //         (
        //             (schema_c.clone(), vec![path_a, path_b, Cow::Owned(PathBuf::from("item_c"))]),
        //             hashset![
        //                 Ok((path_a, block_a.clone())),
        //                 Ok((path_b, block_b.clone())),
        //                 Ok((Cow::Owned(PathBuf::from("item_c")), block_c.clone())),
        //             ],
        //         ),
        //         (
        //             (schema_c.clone(), vec![path_a, path_b]),
        //             hashset![
        //                 Ok((path_a, block_a.clone())),
        //                 Ok((path_b, block_b.clone())),
        //                 Err(Error::UnusedTaggedBlock(block_c.clone(), str!("item_c"))),
        //             ],
        //         ),
        //         (
        //             (schema_c.clone(), vec![path_a, path_b, Cow::Owned(PathBuf::from("item_c")), Cow::Owned(PathBuf::from("item_d"))]),
        //             hashset![
        //                 Ok((path_a, block_a.clone())),
        //                 Ok((path_b, block_b.clone())),
        //                 Ok((Cow::Owned(PathBuf::from("item_c")), block_c.clone())),
        //                 Err(Error::UnusedItemPath(PathBuf::from("item_d"))),
        //             ],
        //         ),
        //         (
        //             (schema_c.clone(), vec![path_a, path_b, Cow::Owned(PathBuf::from("item_d"))]),
        //             hashset![
        //                 Ok((path_a, block_a.clone())),
        //                 Ok((path_b, block_b.clone())),
        //                 Err(Error::UnusedTaggedBlock(block_c.clone(), str!("item_c"))),
        //                 Err(Error::UnusedItemPath(PathBuf::from("item_d"))),
        //             ],
        //         ),
        //     ];

        //     for (input, expected) in inputs_and_expected {
        //         let (meta_schema, item_paths) = input;
        //         let produced = Plexer::new(meta_schema, item_paths.into_iter()).collect::<HashSet<_>>();
        //         assert_eq!(expected, produced);
        //     }
    }
}
