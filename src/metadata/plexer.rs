
use std::path::PathBuf;

use crate::config::sorter::Sorter;
use crate::metadata::types::MetaBlock;
use crate::metadata::types::MetaBlockMap;
use crate::metadata::types::MetaStructure;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Error {
    UnusedItemPath(PathBuf),
    UnusedMetaBlock(MetaBlock, Option<String>),
    NamelessItemPath(PathBuf),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::UnusedItemPath(ref p) => write!(f, "item path was unused in plexing: {}", p.display()),
            Error::UnusedMetaBlock(_, ref opt_tag) => {
                let tag_desc = match opt_tag {
                    Some(tag) => format!(", with tag: {}", tag),
                    None => String::from(""),
                };

                write!(f, "meta block was unused in plexing{}", tag_desc)
            },
            Error::NamelessItemPath(ref p) => write!(f, "item path did not have a file name: {}", p.display()),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Error::UnusedItemPath(..) => None,
            Error::UnusedMetaBlock(..) => None,
            Error::NamelessItemPath(..) => None,
        }
    }
}

pub enum MetaPlexer<I: Iterator<Item = PathBuf>> {
    One(Option<MetaBlock>, I),
    Seq(std::vec::IntoIter<MetaBlock>, std::vec::IntoIter<PathBuf>),
    Map(MetaBlockMap, I),
}

impl<I> Iterator for MetaPlexer<I>
where
    I: Iterator<Item = PathBuf>,
{
    type Item = Result<(PathBuf, MetaBlock), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::One(ref mut opt_block, ref mut path_iter) => {
                match (opt_block.take(), path_iter.next()) {
                    // Both iterators are exhausted, so this one is as well.
                    (None, None) => None,

                    // Both iterators produced a result, emit a successful plex result.
                    (Some(block), Some(path)) => Some(Ok((path, block))),

                    // Got a file path with no meta block, report an error.
                    (None, Some(path)) => Some(Err(Error::UnusedItemPath(path))),

                    // Got a meta block with no file path, report an error.
                    (Some(block), None) => Some(Err(Error::UnusedMetaBlock(block, None))),
                }
            },
            Self::Seq(ref mut block_iter, ref mut sorted_path_iter) => {
                match (block_iter.next(), sorted_path_iter.next()) {
                    // Both iterators are exhausted, so this one is as well.
                    (None, None) => None,

                    // Both iterators produced a result, emit a successful plex result.
                    (Some(block), Some(path)) => Some(Ok((path, block))),

                    // Got a file path with no meta block, report an error.
                    (None, Some(path)) => Some(Err(Error::UnusedItemPath(path))),

                    // Got a meta block with no file path, report an error.
                    (Some(block), None) => Some(Err(Error::UnusedMetaBlock(block, None))),
                }
            },
            Self::Map(ref mut block_mapping, ref mut path_iter) => {
                match path_iter.next() {
                    Some(path) => {
                        // Try and obtain a file name from the path, and convert into a string for lookup.
                        // If this fails, return an error for this iteration and then skip the string.
                        match path.file_name().and_then(|os| os.to_str()) {
                            None => Some(Err(Error::NamelessItemPath(path))),
                            Some(file_name_str) => {
                                // See if the file name string is in the meta block mapping.
                                match block_mapping.swap_remove(file_name_str) {
                                    // No meta block in the mapping had a matching file name, report an error.
                                    None => Some(Err(Error::UnusedItemPath(path))),

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
                            Some((block_tag, block)) => Some(Err(Error::UnusedMetaBlock(block, Some(block_tag)))),

                            // No more meta blocks were found, this iterator is now exhausted.
                            None => None,
                        }
                    },
                }
            },
        }
    }
}

impl<I: Iterator<Item = PathBuf>> std::iter::FusedIterator for MetaPlexer<I> {}

impl<I: Iterator<Item = PathBuf>> MetaPlexer<I> {
    pub fn new(meta_structure: MetaStructure, file_path_iter: I, sorter: Sorter) -> Self {
        match meta_structure {
            MetaStructure::One(mb) => Self::One(Some(mb), file_path_iter),
            MetaStructure::Seq(mb_seq) => {
                // Need to pre-sort the file paths.
                let mut file_paths = file_path_iter.collect::<Vec<_>>();
                file_paths.sort_by(|a, b| sorter.path_sort_cmp(a, b));

                Self::Seq(mb_seq.into_iter(), file_paths.into_iter())
            },
            MetaStructure::Map(mb_map) => Self::Map(mb_map, file_path_iter),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MetaPlexer;
    use super::Error;

    use std::path::PathBuf;
    use std::collections::HashSet;

    use crate::config::sorter::Sorter;
    use crate::metadata::types::MetaStructure;
    use crate::metadata::types::MetaVal;

    #[test]
    fn test_plex() {
        let mb_a = btreemap![
            String::from("key_1a") => MetaVal::String(String::from("val_1a")),
            String::from("key_1b") => MetaVal::String(String::from("val_1b")),
            String::from("key_1c") => MetaVal::String(String::from("val_1c")),
        ];
        let mb_b = btreemap![
            String::from("key_2a") => MetaVal::String(String::from("val_2a")),
            String::from("key_2b") => MetaVal::String(String::from("val_2b")),
            String::from("key_2c") => MetaVal::String(String::from("val_2c")),
        ];
        let mb_c = btreemap![
            String::from("key_3a") => MetaVal::String(String::from("val_3a")),
            String::from("key_3b") => MetaVal::String(String::from("val_3b")),
            String::from("key_3c") => MetaVal::String(String::from("val_3c")),
        ];

        let ms_a = MetaStructure::One(mb_a.clone());
        let ms_b = MetaStructure::Seq(vec![mb_a.clone(), mb_b.clone(), mb_c.clone()]);
        let ms_c = MetaStructure::Map(indexmap![
            String::from("item_c") => mb_c.clone(),
            String::from("item_a") => mb_a.clone(),
            String::from("item_b") => mb_b.clone(),
        ]);

        // Test single and sequence structures.
        let inputs_and_expected = vec![
            (
                (ms_a.clone(), vec![PathBuf::from("item_a")]),
                vec![
                    Ok((PathBuf::from("item_a"), mb_a.clone())),
                ],
            ),
            (
                (ms_a.clone(), vec![PathBuf::from("item_a"), PathBuf::from("item_b")]),
                vec![
                    Ok((PathBuf::from("item_a"), mb_a.clone())),
                    Err(Error::UnusedItemPath(PathBuf::from("item_b"))),
                ],
            ),
            (
                (ms_a.clone(), vec![]),
                vec![
                    Err(Error::UnusedMetaBlock(mb_a.clone(), None)),
                ],
            ),
            (
                (ms_b.clone(), vec![PathBuf::from("item_a"), PathBuf::from("item_b"), PathBuf::from("item_c")]),
                vec![
                    Ok((PathBuf::from("item_a"), mb_a.clone())),
                    Ok((PathBuf::from("item_b"), mb_b.clone())),
                    Ok((PathBuf::from("item_c"), mb_c.clone())),
                ],
            ),
            (
                (ms_b.clone(), vec![PathBuf::from("item_a"), PathBuf::from("item_b"), PathBuf::from("item_c"), PathBuf::from("item_d")]),
                vec![
                    Ok((PathBuf::from("item_a"), mb_a.clone())),
                    Ok((PathBuf::from("item_b"), mb_b.clone())),
                    Ok((PathBuf::from("item_c"), mb_c.clone())),
                    Err(Error::UnusedItemPath(PathBuf::from("item_d"))),
                ],
            ),
            (
                (ms_b.clone(), vec![PathBuf::from("item_a")]),
                vec![
                    Ok((PathBuf::from("item_a"), mb_a.clone())),
                    Err(Error::UnusedMetaBlock(mb_b.clone(), None)),
                    Err(Error::UnusedMetaBlock(mb_c.clone(), None)),
                ],
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let (meta_structure, item_paths) = input;
            let produced = MetaPlexer::new(meta_structure, item_paths.into_iter(), Sorter::default()).collect::<Vec<_>>();
            assert_eq!(expected, produced);
        }

        // Test mapping structures.
        let inputs_and_expected = vec![
            (
                (ms_c.clone(), vec![PathBuf::from("item_a"), PathBuf::from("item_b"), PathBuf::from("item_c")]),
                hashset![
                    Ok((PathBuf::from("item_a"), mb_a.clone())),
                    Ok((PathBuf::from("item_b"), mb_b.clone())),
                    Ok((PathBuf::from("item_c"), mb_c.clone())),
                ],
            ),
            (
                (ms_c.clone(), vec![PathBuf::from("item_a"), PathBuf::from("item_b")]),
                hashset![
                    Ok((PathBuf::from("item_a"), mb_a.clone())),
                    Ok((PathBuf::from("item_b"), mb_b.clone())),
                    Err(Error::UnusedMetaBlock(mb_c.clone(), Some(String::from("item_c")))),
                ],
            ),
            (
                (ms_c.clone(), vec![PathBuf::from("item_a"), PathBuf::from("item_b"), PathBuf::from("item_c"), PathBuf::from("item_d")]),
                hashset![
                    Ok((PathBuf::from("item_a"), mb_a.clone())),
                    Ok((PathBuf::from("item_b"), mb_b.clone())),
                    Ok((PathBuf::from("item_c"), mb_c.clone())),
                    Err(Error::UnusedItemPath(PathBuf::from("item_d"))),
                ],
            ),
            (
                (ms_c.clone(), vec![PathBuf::from("item_a"), PathBuf::from("item_b"), PathBuf::from("item_d")]),
                hashset![
                    Ok((PathBuf::from("item_a"), mb_a.clone())),
                    Ok((PathBuf::from("item_b"), mb_b.clone())),
                    Err(Error::UnusedMetaBlock(mb_c.clone(), Some(String::from("item_c")))),
                    Err(Error::UnusedItemPath(PathBuf::from("item_d"))),
                ],
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let (meta_structure, item_paths) = input;
            let produced = MetaPlexer::new(meta_structure, item_paths.into_iter(), Sorter::default()).collect::<HashSet<_>>();
            assert_eq!(expected, produced);
        }
    }
}
