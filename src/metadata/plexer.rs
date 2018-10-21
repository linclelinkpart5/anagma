use std::path::Path;
use std::path::PathBuf;
use std::collections::HashMap;

use itertools::Itertools;
use itertools::EitherOrBoth;

use library::sort_order::SortOrder;
use metadata::structure::MetaStructure;
use metadata::types::MetaBlock;
use util::GenConverter;

#[derive(Fail, Debug, PartialEq, Eq, Hash)]
pub enum PlexItemError {
    #[fail(display = "item path was unused in plexing: {:?}", _0)]
    UnusedItemPath(PathBuf),
    #[fail(display = "meta block was unused in plexing")]
    UnusedMetaBlock(MetaBlock, Option<String>),
    #[fail(display = "item path did not have a file name: {:?}", _0)]
    NamelessItemPath(PathBuf),
}

pub struct MetaPlexer;

impl MetaPlexer {
    pub fn plex<II, P>(
        meta_structure: MetaStructure,
        item_paths: II,
        sort_order: SortOrder,
    ) -> impl Iterator<Item = Result<(PathBuf, MetaBlock), PlexItemError>>
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
                            yield Err(PlexItemError::UnusedItemPath(excess_item_path.as_ref().to_path_buf()));
                        }
                    }
                    else {
                        yield Err(PlexItemError::UnusedMetaBlock(meta_block, None));
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
                                yield Err(PlexItemError::UnusedItemPath(item_path.as_ref().to_path_buf()));
                            },
                            EitherOrBoth::Right(meta_block) => {
                                yield Err(PlexItemError::UnusedMetaBlock(meta_block, None));
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
                                yield Err(PlexItemError::NamelessItemPath(item_path.as_ref().to_path_buf()));
                                continue;
                            },
                        };

                        match meta_block_map.remove(&key) {
                            Some(meta_block) => {
                                yield Ok((item_path.as_ref().to_path_buf(), meta_block));
                            },
                            None => {
                                // Key was not found, encountered a file that was not tagged in the mapping.
                                yield Err(PlexItemError::UnusedItemPath(item_path.as_ref().to_path_buf()));
                                continue;
                            },
                        };
                    }

                    // Warn for any leftover map entries.
                    for (k, mb) in meta_block_map.into_iter() {
                        yield Err(PlexItemError::UnusedMetaBlock(mb, Some(k)));
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
    use super::PlexItemError;

    use std::path::Path;
    use std::path::PathBuf;
    use std::collections::HashSet;

    use library::sort_order::SortOrder;
    use metadata::structure::MetaStructure;
    use metadata::types::val::MetaVal;

    #[test]
    fn test_plex() {
        let mb_a = btreemap![
            String::from("key_1a") => MetaVal::Str(String::from("val_1a")),
            String::from("key_1b") => MetaVal::Str(String::from("val_1b")),
            String::from("key_1c") => MetaVal::Str(String::from("val_1c")),
        ];
        let mb_b = btreemap![
            String::from("key_2a") => MetaVal::Str(String::from("val_2a")),
            String::from("key_2b") => MetaVal::Str(String::from("val_2b")),
            String::from("key_2c") => MetaVal::Str(String::from("val_2c")),
        ];
        let mb_c = btreemap![
            String::from("key_3a") => MetaVal::Str(String::from("val_3a")),
            String::from("key_3b") => MetaVal::Str(String::from("val_3b")),
            String::from("key_3c") => MetaVal::Str(String::from("val_3c")),
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
                    Err(PlexItemError::UnusedItemPath(PathBuf::from("item_b"))),
                ],
            ),
            (
                (ms_a.clone(), vec![]),
                vec![
                    Err(PlexItemError::UnusedMetaBlock(mb_a.clone(), None)),
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
                    Err(PlexItemError::UnusedItemPath(PathBuf::from("item_d"))),
                ],
            ),
            (
                (ms_b.clone(), vec![Path::new("item_a")]),
                vec![
                    Ok((PathBuf::from("item_a"), mb_a.clone())),
                    Err(PlexItemError::UnusedMetaBlock(mb_b.clone(), None)),
                    Err(PlexItemError::UnusedMetaBlock(mb_c.clone(), None)),
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
                    Err(PlexItemError::UnusedMetaBlock(mb_c.clone(), Some(String::from("item_c")))),
                ],
            ),
            (
                (ms_c.clone(), vec![Path::new("item_a"), Path::new("item_b"), Path::new("item_c"), Path::new("item_d")]),
                hashset![
                    Ok((PathBuf::from("item_a"), mb_a.clone())),
                    Ok((PathBuf::from("item_b"), mb_b.clone())),
                    Ok((PathBuf::from("item_c"), mb_c.clone())),
                    Err(PlexItemError::UnusedItemPath(PathBuf::from("item_d"))),
                ],
            ),
            (
                (ms_c.clone(), vec![Path::new("item_a"), Path::new("item_b"), Path::new("item_d")]),
                hashset![
                    Ok((PathBuf::from("item_a"), mb_a.clone())),
                    Ok((PathBuf::from("item_b"), mb_b.clone())),
                    Err(PlexItemError::UnusedMetaBlock(mb_c.clone(), Some(String::from("item_c")))),
                    Err(PlexItemError::UnusedItemPath(PathBuf::from("item_d"))),
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
