//! Methods to assign blocks of metadata to their corresponding item file paths.

use std::path::Path;
use std::path::PathBuf;
use std::iter::FusedIterator;
use std::borrow::Cow;

use crate::metadata::block::Block;
use crate::metadata::block::BlockMapping;
use crate::metadata::schema::Schema;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Error {
    UnusedItemPath(PathBuf),
    UnusedBlock(Block, Option<String>),
    NamelessItemPath(PathBuf),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::UnusedItemPath(ref p) => write!(f, "item path was unused in plexing: {}", p.display()),
            Self::UnusedBlock(_, ref opt_tag) => {
                let tag_desc = match opt_tag {
                    Some(tag) => Cow::Owned(format!(", with tag: {}", tag)),
                    None => Cow::Borrowed(""),
                };

                write!(f, "meta block was unused in plexing{}", tag_desc)
            },
            Self::NamelessItemPath(ref p) => write!(f, "item path did not have a file name: {}", p.display()),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

pub enum Plexer<'a, I>
where
    I: Iterator<Item = Cow<'a, Path>>,
{
    One(Option<Block>, I),
    Seq(std::vec::IntoIter<Block>, I),
    Map(BlockMapping, I),
}

impl<'a, I> Iterator for Plexer<'a, I>
where
    I: Iterator<Item = Cow<'a, Path>>,
{
    type Item = Result<(Cow<'a, Path>, Block), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::One(ref mut opt_block, ref mut path_iter) => {
                match (opt_block.take(), path_iter.next()) {
                    // Both iterators are exhausted, so this one is as well.
                    (None, None) => None,

                    // Both iterators produced a result, emit a successful plex result.
                    (Some(block), Some(path)) => Some(Ok((path, block))),

                    // Got a file path with no meta block, report an error.
                    (None, Some(path)) => Some(Err(Error::UnusedItemPath(path.into()))),

                    // Got a meta block with no file path, report an error.
                    (Some(block), None) => Some(Err(Error::UnusedBlock(block, None))),
                }
            },
            Self::Seq(ref mut block_iter, ref mut sorted_path_iter) => {
                match (block_iter.next(), sorted_path_iter.next()) {
                    // Both iterators are exhausted, so this one is as well.
                    (None, None) => None,

                    // Both iterators produced a result, emit a successful plex result.
                    (Some(block), Some(path)) => Some(Ok((path, block))),

                    // Got a file path with no meta block, report an error.
                    (None, Some(path)) => Some(Err(Error::UnusedItemPath(path.into()))),

                    // Got a meta block with no file path, report an error.
                    (Some(block), None) => Some(Err(Error::UnusedBlock(block, None))),
                }
            },
            Self::Map(ref mut block_mapping, ref mut path_iter) => {
                match path_iter.next() {
                    Some(path) => {
                        // Try and obtain a file name from the path, and convert into a string for lookup.
                        // If this fails, return an error for this iteration and then skip the string.
                        match path.file_name().and_then(|os| os.to_str()) {
                            None => Some(Err(Error::NamelessItemPath(path.into()))),
                            Some(file_name_str) => {
                                // See if the file name string is in the meta block mapping.
                                match block_mapping.swap_remove(file_name_str) {
                                    // No meta block in the mapping had a matching file name, report an error.
                                    None => Some(Err(Error::UnusedItemPath(path.into()))),

                                    // Found a matching meta block, emit a successful plex result.
                                    Some(block) => Some(Ok((path, block))),
                                }
                            },
                        }
                    },
                    None => {
                        // No more file paths, see if there are any more meta blocks.
                        match block_mapping.pop() {
                            // Found an orphaned meta block, report an error.
                            Some((block_tag, block)) => Some(Err(Error::UnusedBlock(block, Some(block_tag)))),

                            // No more meta blocks were found, this iterator is now exhausted.
                            None => None,
                        }
                    },
                }
            },
        }
    }
}

impl<'a, I> FusedIterator for Plexer<'a, I>
where
    I: Iterator<Item = Cow<'a, Path>>,
{}

impl<'a, I> Plexer<'a, I>
where
    I: Iterator<Item = Cow<'a, Path>>,
{
    /// Creates a new `Plexer`.
    pub fn new(schema: Schema, file_path_iter: I) -> Self {
        match schema {
            Schema::One(mb) => Self::One(Some(mb), file_path_iter),
            Schema::Seq(mb_seq) => Self::Seq(mb_seq.into_iter(), file_path_iter),
            Schema::Map(mb_map) => Self::Map(mb_map, file_path_iter),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::HashSet;

    use maplit::{hashset, btreemap};
    use indexmap::indexmap;

    use crate::test_util::TestUtil as TU;

    #[test]
    fn plex() {
        let block_a = btreemap![
            String::from("key_1a") => TU::s("val_1a"),
            String::from("key_1b") => TU::s("val_1b"),
            String::from("key_1c") => TU::s("val_1c"),
        ];
        let block_b = btreemap![
            String::from("key_2a") => TU::s("val_2a"),
            String::from("key_2b") => TU::s("val_2b"),
            String::from("key_2c") => TU::s("val_2c"),
        ];
        let block_c = btreemap![
            String::from("key_3a") => TU::s("val_3a"),
            String::from("key_3b") => TU::s("val_3b"),
            String::from("key_3c") => TU::s("val_3c"),
        ];

        let structure_a = Schema::One(block_a.clone());
        let structure_b = Schema::Seq(vec![block_a.clone(), block_b.clone(), block_c.clone()]);
        let structure_c = Schema::Map(indexmap![
            String::from("item_c") => block_c.clone(),
            String::from("item_a") => block_a.clone(),
            String::from("item_b") => block_b.clone(),
        ]);

        // Test single and sequence structures.
        let inputs_and_expected = vec![
            (
                (structure_a.clone(), vec![Cow::Owned(PathBuf::from("item_a"))]),
                vec![
                    Ok((Cow::Owned(PathBuf::from("item_a")), block_a.clone())),
                ],
            ),
            (
                (structure_a.clone(), vec![Cow::Owned(PathBuf::from("item_a")), Cow::Owned(PathBuf::from("item_b"))]),
                vec![
                    Ok((Cow::Owned(PathBuf::from("item_a")), block_a.clone())),
                    Err(Error::UnusedItemPath(PathBuf::from("item_b"))),
                ],
            ),
            (
                (structure_a.clone(), vec![]),
                vec![
                    Err(Error::UnusedBlock(block_a.clone(), None)),
                ],
            ),
            (
                (structure_b.clone(), vec![Cow::Owned(PathBuf::from("item_a")), Cow::Owned(PathBuf::from("item_b")), Cow::Owned(PathBuf::from("item_c"))]),
                vec![
                    Ok((Cow::Owned(PathBuf::from("item_a")), block_a.clone())),
                    Ok((Cow::Owned(PathBuf::from("item_b")), block_b.clone())),
                    Ok((Cow::Owned(PathBuf::from("item_c")), block_c.clone())),
                ],
            ),
            (
                (structure_b.clone(), vec![Cow::Owned(PathBuf::from("item_a")), Cow::Owned(PathBuf::from("item_b")), Cow::Owned(PathBuf::from("item_c")), Cow::Owned(PathBuf::from("item_d"))]),
                vec![
                    Ok((Cow::Owned(PathBuf::from("item_a")), block_a.clone())),
                    Ok((Cow::Owned(PathBuf::from("item_b")), block_b.clone())),
                    Ok((Cow::Owned(PathBuf::from("item_c")), block_c.clone())),
                    Err(Error::UnusedItemPath(PathBuf::from("item_d"))),
                ],
            ),
            (
                (structure_b.clone(), vec![Cow::Owned(PathBuf::from("item_a"))]),
                vec![
                    Ok((Cow::Owned(PathBuf::from("item_a")), block_a.clone())),
                    Err(Error::UnusedBlock(block_b.clone(), None)),
                    Err(Error::UnusedBlock(block_c.clone(), None)),
                ],
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let (meta_structure, item_paths) = input;
            let produced = Plexer::new(meta_structure, item_paths.into_iter()).collect::<Vec<_>>();
            assert_eq!(expected, produced);
        }

        // Test mapping structures.
        let inputs_and_expected = vec![
            (
                (structure_c.clone(), vec![Cow::Owned(PathBuf::from("item_a")), Cow::Owned(PathBuf::from("item_b")), Cow::Owned(PathBuf::from("item_c"))]),
                hashset![
                    Ok((Cow::Owned(PathBuf::from("item_a")), block_a.clone())),
                    Ok((Cow::Owned(PathBuf::from("item_b")), block_b.clone())),
                    Ok((Cow::Owned(PathBuf::from("item_c")), block_c.clone())),
                ],
            ),
            (
                (structure_c.clone(), vec![Cow::Owned(PathBuf::from("item_a")), Cow::Owned(PathBuf::from("item_b"))]),
                hashset![
                    Ok((Cow::Owned(PathBuf::from("item_a")), block_a.clone())),
                    Ok((Cow::Owned(PathBuf::from("item_b")), block_b.clone())),
                    Err(Error::UnusedBlock(block_c.clone(), Some(String::from("item_c")))),
                ],
            ),
            (
                (structure_c.clone(), vec![Cow::Owned(PathBuf::from("item_a")), Cow::Owned(PathBuf::from("item_b")), Cow::Owned(PathBuf::from("item_c")), Cow::Owned(PathBuf::from("item_d"))]),
                hashset![
                    Ok((Cow::Owned(PathBuf::from("item_a")), block_a.clone())),
                    Ok((Cow::Owned(PathBuf::from("item_b")), block_b.clone())),
                    Ok((Cow::Owned(PathBuf::from("item_c")), block_c.clone())),
                    Err(Error::UnusedItemPath(PathBuf::from("item_d"))),
                ],
            ),
            (
                (structure_c.clone(), vec![Cow::Owned(PathBuf::from("item_a")), Cow::Owned(PathBuf::from("item_b")), Cow::Owned(PathBuf::from("item_d"))]),
                hashset![
                    Ok((Cow::Owned(PathBuf::from("item_a")), block_a.clone())),
                    Ok((Cow::Owned(PathBuf::from("item_b")), block_b.clone())),
                    Err(Error::UnusedBlock(block_c.clone(), Some(String::from("item_c")))),
                    Err(Error::UnusedItemPath(PathBuf::from("item_d"))),
                ],
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let (meta_structure, item_paths) = input;
            let produced = Plexer::new(meta_structure, item_paths.into_iter()).collect::<HashSet<_>>();
            assert_eq!(expected, produced);
        }
    }
}
