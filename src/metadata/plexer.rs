use std::path::Path;
use std::path::PathBuf;

use itertools::Itertools;
use itertools::EitherOrBoth;

use config::sort_order::SortOrder;
use metadata::types::MetaStructure;
use metadata::types::MetaBlock;
use util::GenConverter;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Error<'k> {
    UnusedItemPath(PathBuf),
    UnusedMetaBlock(MetaBlock<'k>, Option<String>),
    NamelessItemPath(PathBuf),
}

impl<'k> std::fmt::Display for Error<'k> {
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

impl<'k> std::error::Error for Error<'k> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Error::UnusedItemPath(..) => None,
            Error::UnusedMetaBlock(..) => None,
            Error::NamelessItemPath(..) => None,
        }
    }
}

pub struct MetaPlexer;

impl MetaPlexer {
    pub fn plex<'k, II, P>(
        meta_structure: MetaStructure<'k>,
        item_paths: II,
        sort_order: SortOrder,
    ) -> impl Iterator<Item = Result<(PathBuf, MetaBlock<'k>), Error>>
    where II: IntoIterator<Item = P>,
          P: AsRef<Path>,
    {
        let closure = move || {
            let mut item_paths = item_paths.into_iter();

            match meta_structure {
                MetaStructure::One(meta_block) => {
                    // Exactly one item path is expected.
                    if let Some(item_path) = item_paths.next() {
                        yield Ok((item_path.as_ref().to_path_buf(), meta_block));

                        // If there are excess paths provided, error for each of them.
                        for excess_item_path in item_paths {
                            yield Err(Error::UnusedItemPath(excess_item_path.as_ref().to_path_buf()));
                        }
                    }
                    else {
                        yield Err(Error::UnusedMetaBlock(meta_block, None));
                    }
                },
                MetaStructure::Seq(meta_block_seq) => {
                    let mut item_paths: Vec<_> = item_paths.into_iter().collect();

                    // Need to sort in order to get correct matches.
                    item_paths.sort_by(|a, b| sort_order.path_sort_cmp(a, b));

                    for eob in item_paths.into_iter().zip_longest(meta_block_seq) {
                        match eob {
                            EitherOrBoth::Both(item_path, meta_block) => {
                                yield Ok((item_path.as_ref().to_path_buf(), meta_block));
                            },
                            EitherOrBoth::Left(item_path) => {
                                yield Err(Error::UnusedItemPath(item_path.as_ref().to_path_buf()));
                            },
                            EitherOrBoth::Right(meta_block) => {
                                yield Err(Error::UnusedMetaBlock(meta_block, None));
                            },
                        }
                    }
                },
                // TODO: See if there is a way to do this without mutating the original value.
                MetaStructure::Map(mut meta_block_map) => {
                    for item_path in item_paths {
                        // Use the file name of the item path as a key into the mapping.
                        // LEARN: The string clone is due to a limitation of references, none can be alive during a yield.
                        let key = match item_path.as_ref().file_name().and_then(|os| os.to_str()).map(|s| s.to_string()) {
                            Some(file_name) => file_name,
                            None => {
                                yield Err(Error::NamelessItemPath(item_path.as_ref().to_path_buf()));
                                continue;
                            },
                        };

                        match meta_block_map.remove(&key) {
                            Some(meta_block) => {
                                yield Ok((item_path.as_ref().to_path_buf(), meta_block));
                            },
                            None => {
                                // Key was not found, encountered a file that was not tagged in the mapping.
                                yield Err(Error::UnusedItemPath(item_path.as_ref().to_path_buf()));
                                continue;
                            },
                        };
                    }

                    // Warn for any leftover map entries.
                    for (k, mb) in meta_block_map.into_iter() {
                        yield Err(Error::UnusedMetaBlock(mb, Some(k)));
                    }
                },
            };
        };

        GenConverter::gen_to_iter(closure)
    }
}

#[cfg(test)]
mod tests {
    use super::MetaPlexer;
    use super::Error;

    use std::path::Path;
    use std::path::PathBuf;
    use std::collections::HashSet;

    use config::sort_order::SortOrder;
    use metadata::types::MetaStructure;
    use metadata::types::MetaVal;
    use metadata::types::MetaKey;

    #[test]
    fn test_plex() {
        let mb_a = btreemap![
            MetaKey::from("key_1a") => MetaVal::Str(String::from("val_1a")),
            MetaKey::from("key_1b") => MetaVal::Str(String::from("val_1b")),
            MetaKey::from("key_1c") => MetaVal::Str(String::from("val_1c")),
        ];
        let mb_b = btreemap![
            MetaKey::from("key_2a") => MetaVal::Str(String::from("val_2a")),
            MetaKey::from("key_2b") => MetaVal::Str(String::from("val_2b")),
            MetaKey::from("key_2c") => MetaVal::Str(String::from("val_2c")),
        ];
        let mb_c = btreemap![
            MetaKey::from("key_3a") => MetaVal::Str(String::from("val_3a")),
            MetaKey::from("key_3b") => MetaVal::Str(String::from("val_3b")),
            MetaKey::from("key_3c") => MetaVal::Str(String::from("val_3c")),
        ];

        let ms_a = MetaStructure::One(mb_a.clone());
        let ms_b = MetaStructure::Seq(vec![mb_a.clone(), mb_b.clone(), mb_c.clone()]);
        let ms_c = MetaStructure::Map(hashmap![
            String::from("item_c") => mb_c.clone(),
            String::from("item_a") => mb_a.clone(),
            String::from("item_b") => mb_b.clone(),
        ]);

        // Test single and sequence structures.
        let inputs_and_expected = vec![
            (
                (ms_a.clone(), vec![Path::new("item_a")]),
                vec![
                    Ok((PathBuf::from("item_a"), mb_a.clone())),
                ],
            ),
            (
                (ms_a.clone(), vec![Path::new("item_a"), Path::new("item_b")]),
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
                (ms_b.clone(), vec![Path::new("item_a"), Path::new("item_b"), Path::new("item_c")]),
                vec![
                    Ok((PathBuf::from("item_a"), mb_a.clone())),
                    Ok((PathBuf::from("item_b"), mb_b.clone())),
                    Ok((PathBuf::from("item_c"), mb_c.clone())),
                ],
            ),
            (
                (ms_b.clone(), vec![Path::new("item_a"), Path::new("item_b"), Path::new("item_c"), Path::new("item_d")]),
                vec![
                    Ok((PathBuf::from("item_a"), mb_a.clone())),
                    Ok((PathBuf::from("item_b"), mb_b.clone())),
                    Ok((PathBuf::from("item_c"), mb_c.clone())),
                    Err(Error::UnusedItemPath(PathBuf::from("item_d"))),
                ],
            ),
            (
                (ms_b.clone(), vec![Path::new("item_a")]),
                vec![
                    Ok((PathBuf::from("item_a"), mb_a.clone())),
                    Err(Error::UnusedMetaBlock(mb_b.clone(), None)),
                    Err(Error::UnusedMetaBlock(mb_c.clone(), None)),
                ],
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let (meta_structure, item_paths) = input;
            let produced: Vec<_> = MetaPlexer::plex(meta_structure, item_paths, SortOrder::Name).collect();
            assert_eq!(expected, produced);
        }

        // Test mapping structures.
        let inputs_and_expected = vec![
            (
                (ms_c.clone(), vec![Path::new("item_a"), Path::new("item_b"), Path::new("item_c")]),
                hashset![
                    Ok((PathBuf::from("item_a"), mb_a.clone())),
                    Ok((PathBuf::from("item_b"), mb_b.clone())),
                    Ok((PathBuf::from("item_c"), mb_c.clone())),
                ],
            ),
            (
                (ms_c.clone(), vec![Path::new("item_a"), Path::new("item_b")]),
                hashset![
                    Ok((PathBuf::from("item_a"), mb_a.clone())),
                    Ok((PathBuf::from("item_b"), mb_b.clone())),
                    Err(Error::UnusedMetaBlock(mb_c.clone(), Some(String::from("item_c")))),
                ],
            ),
            (
                (ms_c.clone(), vec![Path::new("item_a"), Path::new("item_b"), Path::new("item_c"), Path::new("item_d")]),
                hashset![
                    Ok((PathBuf::from("item_a"), mb_a.clone())),
                    Ok((PathBuf::from("item_b"), mb_b.clone())),
                    Ok((PathBuf::from("item_c"), mb_c.clone())),
                    Err(Error::UnusedItemPath(PathBuf::from("item_d"))),
                ],
            ),
            (
                (ms_c.clone(), vec![Path::new("item_a"), Path::new("item_b"), Path::new("item_d")]),
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
            let produced: HashSet<_> = MetaPlexer::plex(meta_structure, item_paths, SortOrder::Name).collect();
            assert_eq!(expected, produced);
        }
    }
}
