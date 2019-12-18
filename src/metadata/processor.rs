use std::path::Path;
use std::path::PathBuf;
use std::collections::HashMap;

use crate::config::selection::Selection;
use crate::config::sorter::Sorter;
use crate::config::serialize_format::SerializeFormat;
use crate::metadata::block::Block;
use crate::metadata::location::MetaLocation;
use crate::metadata::location::Error as LocationError;
use crate::metadata::reader::Error as ReaderError;
use crate::metadata::plexer::MetaPlexer;
use crate::metadata::reader::MetaReader;

#[derive(Debug)]
pub enum Error {
    CannotReadMetadata(ReaderError),
    CannotFindItemPaths(LocationError),
    CannotFindMetaPath(LocationError),
    MissingMetadata,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::CannotReadMetadata(ref err) => write!(f, "cannot read metadata file: {}", err),
            Error::CannotFindItemPaths(ref err) => write!(f, "cannot find item file paths: {}", err),
            Error::CannotFindMetaPath(ref err) => write!(f, "cannot find meta file path: {}", err),
            Error::MissingMetadata => write!(f, "missing metadata"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Error::CannotReadMetadata(ref err) => Some(err),
            Error::CannotFindItemPaths(ref err) => Some(err),
            Error::CannotFindMetaPath(ref err) => Some(err),
            Error::MissingMetadata => None,
        }
    }
}

const META_LOCATION_ORDER: &[MetaLocation] = &[MetaLocation::Siblings, MetaLocation::Contains];

pub struct MetaProcessor;

impl MetaProcessor {
    pub fn process_meta_file<P>(
        meta_path: P,
        meta_location: MetaLocation,
        serialize_format: SerializeFormat,
        selection: &Selection,
        sorter: Sorter,
    ) -> Result<HashMap<PathBuf, Block>, Error>
    where
        P: AsRef<Path>,
    {
        let meta_structure = serialize_format.from_file(&meta_path, meta_location).map_err(Error::CannotReadMetadata)?;

        let selected_item_paths = meta_location.get_selected_item_paths(&meta_path, selection).map_err(Error::CannotFindItemPaths)?;

        let mut meta_plexed = hashmap![];

        let meta_plexer = MetaPlexer::new(meta_structure, selected_item_paths.into_iter(), sorter);

        for meta_plex_res in meta_plexer {
            match meta_plex_res {
                Ok((item_path, mb)) => { meta_plexed.insert(item_path, mb); },
                Err(e) => { warn!("{}", e); },
            }
        }

        Ok(meta_plexed)
    }

    // Processes metadata for an item file.
    // This performs the necessary merging of all metadata from different targets for this one item file.
    // Merging is "combine-last", so matching result keys for subsequent locations override earlier keys.
    pub fn process_item_file<P>(
        item_path: P,
        serialize_format: SerializeFormat,
        selection: &Selection,
        sorter: Sorter,
    ) -> Result<Block, Error>
    where
        P: AsRef<Path>,
    {
        let mut comp_mb = Block::new();

        for meta_location in META_LOCATION_ORDER.into_iter() {
            let meta_path = match meta_location.get_meta_path(&item_path, serialize_format) {
                Err(e) => {
                    match e {
                        LocationError::NonexistentMetaPath(..) |
                            LocationError::InvalidItemDirPath(..) |
                            LocationError::NoItemPathParent(..) => { continue; },
                        _ => { return Err(e).map_err(Error::CannotFindMetaPath)?; }
                    }
                },
                Ok(p) => p,
            };

            let mut processed_meta_file = Self::process_meta_file(
                &meta_path,
                *meta_location,
                serialize_format,
                selection,
                sorter,
            )?;

            // The remaining results can be thrown away.
            if let Some(meta_block) = processed_meta_file.remove(item_path.as_ref()) {
                comp_mb.extend(meta_block)
            }
            else {
                Err(Error::MissingMetadata)?
            }
        }

        Ok(comp_mb)
    }
}

#[cfg(test)]
mod tests {
    use super::MetaProcessor;

    use crate::config::Config;
    use crate::config::serialize_format::SerializeFormat;
    use crate::metadata::location::MetaLocation;
    use crate::metadata::value::Value;

    use crate::test_util::create_temp_media_test_dir;

    #[test]
    fn test_process_meta_file() {
        let temp_dir = create_temp_media_test_dir("test_process_meta_file");
        let path = temp_dir.path();

        let config = Config::default();
        let selection = &config.selection;
        let sorter = config.sorter;

        // Success cases
        let inputs_and_expected = vec![
            // (
            //     (path.join("self.yml"), MetaLocation::Contains),
            //     hashmap![
            //         path.to_owned() => btreemap![
            //             "ROOT_self_key".to_owned() => Value::String("ROOT_self_val".to_owned()),
            //             "const_key".to_owned() => Value::String("const_val".to_owned()),
            //             "self_key".to_owned() => Value::String("self_val".to_owned()),
            //             "overridden".to_owned() => Value::String("ROOT_self".to_owned()),
            //         ],
            //     ],
            // ),
            (
                (path.join("item.yml"), MetaLocation::Siblings),
                hashmap![
                    path.join("ALBUM_01") => btreemap![
                        String::from("ALBUM_01_item_key") => Value::String("ALBUM_01_item_val".to_owned()),
                        String::from("const_key") => Value::String("const_val".to_owned()),
                        String::from("item_key") => Value::String("item_val".to_owned()),
                        String::from("overridden") => Value::String("ALBUM_01_item".to_owned()),
                    ],
                    path.join("ALBUM_02") => btreemap![
                        String::from("ALBUM_02_item_key") => Value::String("ALBUM_02_item_val".to_owned()),
                        String::from("const_key") => Value::String("const_val".to_owned()),
                        String::from("item_key") => Value::String("item_val".to_owned()),
                        String::from("overridden") => Value::String("ALBUM_02_item".to_owned()),
                    ],
                    path.join("ALBUM_03") => btreemap![
                        String::from("ALBUM_03_item_key") => Value::String("ALBUM_03_item_val".to_owned()),
                        String::from("const_key") => Value::String("const_val".to_owned()),
                        String::from("item_key") => Value::String("item_val".to_owned()),
                        String::from("overridden") => Value::String("ALBUM_03_item".to_owned()),
                    ],
                    path.join("ALBUM_04.flac") => btreemap![
                        String::from("ALBUM_04_item_key") => Value::String("ALBUM_04_item_val".to_owned()),
                        String::from("const_key") => Value::String("const_val".to_owned()),
                        String::from("item_key") => Value::String("item_val".to_owned()),
                        String::from("overridden") => Value::String("ALBUM_04_item".to_owned()),
                    ],
                    path.join("ALBUM_05") => btreemap![
                        String::from("ALBUM_05_item_key") => Value::String("ALBUM_05_item_val".to_owned()),
                        String::from("const_key") => Value::String("const_val".to_owned()),
                        String::from("item_key") => Value::String("item_val".to_owned()),
                        String::from("overridden") => Value::String("ALBUM_05_item".to_owned()),
                    ],
                ],
            ),
            // (
            //     (path.join("ALBUM_01").join("self.yml"), MetaLocation::Contains),
            //     hashmap![
            //         path.join("ALBUM_01") => btreemap![
            //             "ALBUM_01_self_key".to_owned() => Value::String("ALBUM_01_self_val".to_owned()),
            //             "const_key".to_owned() => Value::String("const_val".to_owned()),
            //             "self_key".to_owned() => Value::String("self_val".to_owned()),
            //             "overridden".to_owned() => Value::String("ALBUM_01_self".to_owned()),
            //         ],
            //     ],
            // ),
            // (
            //     (path.join("ALBUM_01").join("DISC_01").join("item.yml"), MetaLocation::Siblings),
            //     hashmap![
            //         path.join("ALBUM_01").join("DISC_01").join("TRACK_01.flac") => btreemap![
            //             "TRACK_01_item_key".to_owned() => Value::String("TRACK_01_item_val".to_owned()),
            //             "const_key".to_owned() => Value::String("const_val".to_owned()),
            //             "item_key".to_owned() => Value::String("item_val".to_owned()),
            //             "overridden".to_owned() => Value::String("TRACK_01_item".to_owned()),
            //         ],
            //         path.join("ALBUM_01").join("DISC_01").join("TRACK_02.flac") => btreemap![
            //             "TRACK_02_item_key".to_owned() => Value::String("TRACK_02_item_val".to_owned()),
            //             "const_key".to_owned() => Value::String("const_val".to_owned()),
            //             "item_key".to_owned() => Value::String("item_val".to_owned()),
            //             "overridden".to_owned() => Value::String("TRACK_02_item".to_owned()),
            //         ],
            //         path.join("ALBUM_01").join("DISC_01").join("TRACK_03.flac") => btreemap![
            //             "TRACK_03_item_key".to_owned() => Value::String("TRACK_03_item_val".to_owned()),
            //             "const_key".to_owned() => Value::String("const_val".to_owned()),
            //             "item_key".to_owned() => Value::String("item_val".to_owned()),
            //             "overridden".to_owned() => Value::String("TRACK_03_item".to_owned()),
            //         ],
            //     ],
            // ),
        ];

        for (input, expected) in inputs_and_expected {
            let (meta_path, meta_location) = input;

            let produced = MetaProcessor::process_meta_file(meta_path, meta_location, SerializeFormat::Yaml, selection, sorter).unwrap();
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_process_item_file() {
        let temp_dir = create_temp_media_test_dir("test_process_item_file");
        let path = temp_dir.path();

        let config = Config::default();
        let selection = &config.selection;
        let sorter = config.sorter;

        // Success cases
        let inputs_and_expected = vec![
            (
                path.to_owned(),
                btreemap![
                    String::from("ROOT_self_key") => Value::String("ROOT_self_val".to_owned()),
                    String::from("const_key") => Value::String("const_val".to_owned()),
                    String::from("self_key") => Value::String("self_val".to_owned()),
                    String::from("overridden") => Value::String("ROOT_self".to_owned()),
                ],
            ),
            (
                path.join("ALBUM_01"),
                btreemap![
                    String::from("ALBUM_01_item_key") => Value::String("ALBUM_01_item_val".to_owned()),
                    String::from("ALBUM_01_self_key") => Value::String("ALBUM_01_self_val".to_owned()),
                    String::from("const_key") => Value::String("const_val".to_owned()),
                    String::from("item_key") => Value::String("item_val".to_owned()),
                    String::from("self_key") => Value::String("self_val".to_owned()),
                    String::from("overridden") => Value::String("ALBUM_01_self".to_owned()),
                ],
            ),
            (
                path.join("ALBUM_01").join("DISC_01").join("TRACK_01.flac"),
                btreemap![
                    String::from("TRACK_01_item_key") => Value::String("TRACK_01_item_val".to_owned()),
                    String::from("const_key") => Value::String("const_val".to_owned()),
                    String::from("item_key") => Value::String("item_val".to_owned()),
                    String::from("overridden") => Value::String("TRACK_01_item".to_owned()),
                ],
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let item_path = input;

            let produced = MetaProcessor::process_item_file(item_path, SerializeFormat::Yaml, selection, sorter).unwrap();
            assert_eq!(expected, produced);
        }
    }
}
