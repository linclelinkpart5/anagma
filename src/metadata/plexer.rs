use std::path::Path;
use std::path::PathBuf;
use std::collections::HashMap;

use itertools::Itertools;
use itertools::EitherOrBoth;

use metadata::structure::MetaStructure;
use metadata::types::MetaBlock;

pub struct MetaPlexer;

impl MetaPlexer {
    pub fn plex<II, P>(meta_structure: MetaStructure, item_paths: II) -> HashMap<PathBuf, MetaBlock>
    where II: IntoIterator<Item = P>,
          P: AsRef<Path>,
    {
        let mut item_paths = item_paths.into_iter();

        let mut result = hashmap![];

        match meta_structure {
            MetaStructure::One(meta_block) => {
                // Exactly one item path is expected.
                if let Some(item_path) = item_paths.next() {
                    // If there are excess paths provided, warn for each of them.
                    for excess_item_path in item_paths {
                        warn!("unused item path \"{}\"", excess_item_path.as_ref().to_string_lossy());
                    }

                    result.insert(item_path.as_ref().to_path_buf(), meta_block);
                }
                else {
                    warn!("no item paths provided");
                }
            },
            MetaStructure::Seq(meta_block_seq) => {
                for eob in item_paths.zip_longest(meta_block_seq) {
                    match eob {
                        EitherOrBoth::Both(item_path, meta_block) => {
                            result.insert(item_path.as_ref().to_path_buf(), meta_block);
                        },
                        EitherOrBoth::Left(item_path) => {
                            warn!("unused item path \"{}\"", item_path.as_ref().to_string_lossy());
                        },
                        EitherOrBoth::Right(meta_block) => {
                            warn!("unused meta block \"{:?}\"", meta_block);
                        },
                    }
                }
            },
            MetaStructure::Map(mut meta_block_map) => {
                for item_path in item_paths {
                    // Use the file name of the item path as a key into the mapping.
                    let key = match item_path.as_ref().file_name() {
                        Some(file_name) => file_name,
                        None => {
                            warn!("item path does not have a file name");
                            continue;
                        },
                    };

                    match meta_block_map.remove(key) {
                        Some(meta_block) => {
                            result.insert(item_path.as_ref().to_path_buf(), meta_block);
                        },
                        None => {
                            // Key was not found, encountered a file that was not tagged in the mapping.
                            warn!("item file name \"{}\" not found in mapping", key.to_string_lossy());
                            continue;
                        },
                    };
                }

                // Warn for any leftover map entries.
                for (k, _) in meta_block_map.drain() {
                    warn!("key \"{}\" not found in item file paths", k.to_string_lossy());
                }
            },
        };

        result
    }
}

#[cfg(test)]
mod tests {
    use super::MetaPlexer;

    use std::path::Path;
    use std::path::PathBuf;
    use std::ffi::OsString;

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
            OsString::from("item_c.file") => mb_c.clone(),
            OsString::from("item_a.file") => mb_a.clone(),
            OsString::from("item_b.file") => mb_b.clone(),
        ]);

        let inputs_and_expected = vec![
            (
                (ms_a.clone(), vec![Path::new("item_a.file")]),
                hashmap![
                    PathBuf::from("item_a.file") => mb_a.clone(),
                ],
            ),
            (
                (ms_b.clone(), vec![Path::new("item_a.file"), Path::new("item_b.file"), Path::new("item_c.file")]),
                hashmap![
                    PathBuf::from("item_a.file") => mb_a.clone(),
                    PathBuf::from("item_b.file") => mb_b.clone(),
                    PathBuf::from("item_c.file") => mb_c.clone(),
                ],
            ),
            (
                (ms_b.clone(), vec![Path::new("item_a.file"), Path::new("item_b.file"), Path::new("item_c.file"), Path::new("item_d.file")]),
                hashmap![
                    PathBuf::from("item_a.file") => mb_a.clone(),
                    PathBuf::from("item_b.file") => mb_b.clone(),
                    PathBuf::from("item_c.file") => mb_c.clone(),
                ],
            ),
            (
                (ms_b.clone(), vec![Path::new("item_a.file")]),
                hashmap![
                    PathBuf::from("item_a.file") => mb_a.clone(),
                ],
            ),
            (
                (ms_c.clone(), vec![Path::new("item_a.file"), Path::new("item_b.file"), Path::new("item_c.file")]),
                hashmap![
                    PathBuf::from("item_a.file") => mb_a.clone(),
                    PathBuf::from("item_b.file") => mb_b.clone(),
                    PathBuf::from("item_c.file") => mb_c.clone(),
                ],
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let (meta_structure, item_paths) = input;
            let produced = MetaPlexer::plex(meta_structure, item_paths);
            assert_eq!(expected, produced);
        }
    }
}
