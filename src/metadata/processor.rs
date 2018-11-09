use std::path::Path;
use std::path::PathBuf;
use std::collections::HashMap;

use config::selection::Selection;
use config::sort_order::SortOrder;
use config::meta_format::MetaFormat;
use metadata::types::MetaBlock;
use metadata::location::MetaLocation;
use metadata::location::Error as LocationError;
use metadata::reader::Error as ReaderError;
use metadata::plexer::MetaPlexer;
use metadata::reader::MetaReader;

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
        meta_format: MetaFormat,
        selection: &Selection,
        sort_order: SortOrder,
    ) -> Result<HashMap<PathBuf, MetaBlock>, Error>
    where
        P: AsRef<Path>,
    {
        let meta_structure = meta_format.from_file(&meta_path, meta_location).map_err(Error::CannotReadMetadata)?;

        let selected_item_paths = meta_location.get_selected_item_paths(&meta_path, selection).map_err(Error::CannotFindItemPaths)?;

        let mut meta_plexed = hashmap![];
        for meta_plex_res in MetaPlexer::plex(meta_structure, selected_item_paths, sort_order) {
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
        meta_format: MetaFormat,
        selection: &Selection,
        sort_order: SortOrder,
    ) -> Result<MetaBlock, Error>
    where
        P: AsRef<Path>,
    {
        let mut comp_mb = MetaBlock::new();

        for meta_location in META_LOCATION_ORDER.into_iter() {
            let meta_path = match meta_location.get_meta_path(&item_path) {
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

            let mut processed_meta_file = Self::process_meta_file(&meta_path, *meta_location, meta_format, selection, sort_order)?;

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

    // Processes metadata for an item file, and includes all inherited parent metadata.
    pub fn process_item_file_flattened<P>(
        item_path: P,
        meta_format: MetaFormat,
        selection: &Selection,
        sort_order: SortOrder,
    ) -> Result<MetaBlock, Error>
    where
        P: AsRef<Path>,
    {
        let item_path = item_path.as_ref();

        let mut all_mb = MetaBlock::new();
        let ancestors: Vec<_> = item_path.ancestors().collect();

        for parent_dir in ancestors.into_iter().rev() {
            let mut item_mb = Self::process_item_file(parent_dir, meta_format, &selection, sort_order)?;

            all_mb.extend(item_mb);
        }

        Ok(all_mb)
    }
}

#[cfg(test)]
mod tests {
    use super::MetaProcessor;

    use config::Config;
    use config::meta_format::MetaFormat;
    use metadata::location::MetaLocation;
    use metadata::types::MetaVal;

    use test_util::create_temp_media_test_dir;

    #[test]
    fn test_process_meta_file() {
        let temp_dir = create_temp_media_test_dir("test_process_meta_file");
        let path = temp_dir.path();

        let config = Config::default();
        let selection = &config.selection;
        let sort_order = config.sort_order;

        // Success cases
        let inputs_and_expected = vec![
            (
                (path.join("self.yml"), MetaLocation::Contains),
                hashmap![
                    path.to_owned() => btreemap![
                        "ROOT_self_key".to_owned() => MetaVal::Str("ROOT_self_val".to_owned()),
                        "const_key".to_owned() => MetaVal::Str("const_val".to_owned()),
                        "self_key".to_owned() => MetaVal::Str("self_val".to_owned()),
                        "overridden".to_owned() => MetaVal::Str("ROOT_self".to_owned()),
                    ],
                ],
            ),
            (
                (path.join("item.yml"), MetaLocation::Siblings),
                hashmap![
                    path.join("ALBUM_01") => btreemap![
                        "ALBUM_01_item_key".to_owned() => MetaVal::Str("ALBUM_01_item_val".to_owned()),
                        "const_key".to_owned() => MetaVal::Str("const_val".to_owned()),
                        "item_key".to_owned() => MetaVal::Str("item_val".to_owned()),
                        "overridden".to_owned() => MetaVal::Str("ALBUM_01_item".to_owned()),
                    ],
                    path.join("ALBUM_02") => btreemap![
                        "ALBUM_02_item_key".to_owned() => MetaVal::Str("ALBUM_02_item_val".to_owned()),
                        "const_key".to_owned() => MetaVal::Str("const_val".to_owned()),
                        "item_key".to_owned() => MetaVal::Str("item_val".to_owned()),
                        "overridden".to_owned() => MetaVal::Str("ALBUM_02_item".to_owned()),
                    ],
                    path.join("ALBUM_03") => btreemap![
                        "ALBUM_03_item_key".to_owned() => MetaVal::Str("ALBUM_03_item_val".to_owned()),
                        "const_key".to_owned() => MetaVal::Str("const_val".to_owned()),
                        "item_key".to_owned() => MetaVal::Str("item_val".to_owned()),
                        "overridden".to_owned() => MetaVal::Str("ALBUM_03_item".to_owned()),
                    ],
                    path.join("ALBUM_04.flac") => btreemap![
                        "ALBUM_04_item_key".to_owned() => MetaVal::Str("ALBUM_04_item_val".to_owned()),
                        "const_key".to_owned() => MetaVal::Str("const_val".to_owned()),
                        "item_key".to_owned() => MetaVal::Str("item_val".to_owned()),
                        "overridden".to_owned() => MetaVal::Str("ALBUM_04_item".to_owned()),
                    ],
                    path.join("ALBUM_05") => btreemap![
                        "ALBUM_05_item_key".to_owned() => MetaVal::Str("ALBUM_05_item_val".to_owned()),
                        "const_key".to_owned() => MetaVal::Str("const_val".to_owned()),
                        "item_key".to_owned() => MetaVal::Str("item_val".to_owned()),
                        "overridden".to_owned() => MetaVal::Str("ALBUM_05_item".to_owned()),
                    ],
                ],
            ),
            (
                (path.join("ALBUM_01").join("self.yml"), MetaLocation::Contains),
                hashmap![
                    path.join("ALBUM_01") => btreemap![
                        "ALBUM_01_self_key".to_owned() => MetaVal::Str("ALBUM_01_self_val".to_owned()),
                        "const_key".to_owned() => MetaVal::Str("const_val".to_owned()),
                        "self_key".to_owned() => MetaVal::Str("self_val".to_owned()),
                        "overridden".to_owned() => MetaVal::Str("ALBUM_01_self".to_owned()),
                    ],
                ],
            ),
            (
                (path.join("ALBUM_01").join("DISC_01").join("item.yml"), MetaLocation::Siblings),
                hashmap![
                    path.join("ALBUM_01").join("DISC_01").join("TRACK_01.flac") => btreemap![
                        "TRACK_01_item_key".to_owned() => MetaVal::Str("TRACK_01_item_val".to_owned()),
                        "const_key".to_owned() => MetaVal::Str("const_val".to_owned()),
                        "item_key".to_owned() => MetaVal::Str("item_val".to_owned()),
                        "overridden".to_owned() => MetaVal::Str("TRACK_01_item".to_owned()),
                    ],
                    path.join("ALBUM_01").join("DISC_01").join("TRACK_02.flac") => btreemap![
                        "TRACK_02_item_key".to_owned() => MetaVal::Str("TRACK_02_item_val".to_owned()),
                        "const_key".to_owned() => MetaVal::Str("const_val".to_owned()),
                        "item_key".to_owned() => MetaVal::Str("item_val".to_owned()),
                        "overridden".to_owned() => MetaVal::Str("TRACK_02_item".to_owned()),
                    ],
                    path.join("ALBUM_01").join("DISC_01").join("TRACK_03.flac") => btreemap![
                        "TRACK_03_item_key".to_owned() => MetaVal::Str("TRACK_03_item_val".to_owned()),
                        "const_key".to_owned() => MetaVal::Str("const_val".to_owned()),
                        "item_key".to_owned() => MetaVal::Str("item_val".to_owned()),
                        "overridden".to_owned() => MetaVal::Str("TRACK_03_item".to_owned()),
                    ],
                ],
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let (meta_path, meta_location) = input;

            let produced = MetaProcessor::process_meta_file(meta_path, meta_location, MetaFormat::Yaml, selection, sort_order).unwrap();
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_process_item_file() {
        let temp_dir = create_temp_media_test_dir("test_process_item_file");
        let path = temp_dir.path();

        let config = Config::default();
        let selection = &config.selection;
        let sort_order = config.sort_order;

        // Success cases
        let inputs_and_expected = vec![
            (
                path.to_owned(),
                btreemap![
                    "ROOT_self_key".to_owned() => MetaVal::Str("ROOT_self_val".to_owned()),
                    "const_key".to_owned() => MetaVal::Str("const_val".to_owned()),
                    "self_key".to_owned() => MetaVal::Str("self_val".to_owned()),
                    "overridden".to_owned() => MetaVal::Str("ROOT_self".to_owned()),
                ],
            ),
            (
                path.join("ALBUM_01"),
                btreemap![
                    "ALBUM_01_item_key".to_owned() => MetaVal::Str("ALBUM_01_item_val".to_owned()),
                    "ALBUM_01_self_key".to_owned() => MetaVal::Str("ALBUM_01_self_val".to_owned()),
                    "const_key".to_owned() => MetaVal::Str("const_val".to_owned()),
                    "item_key".to_owned() => MetaVal::Str("item_val".to_owned()),
                    "self_key".to_owned() => MetaVal::Str("self_val".to_owned()),
                    "overridden".to_owned() => MetaVal::Str("ALBUM_01_self".to_owned()),
                ],
            ),
            (
                path.join("ALBUM_01").join("DISC_01").join("TRACK_01.flac"),
                btreemap![
                    "TRACK_01_item_key".to_owned() => MetaVal::Str("TRACK_01_item_val".to_owned()),
                    "const_key".to_owned() => MetaVal::Str("const_val".to_owned()),
                    "item_key".to_owned() => MetaVal::Str("item_val".to_owned()),
                    "overridden".to_owned() => MetaVal::Str("TRACK_01_item".to_owned()),
                ],
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let item_path = input;

            let produced = MetaProcessor::process_item_file(item_path, MetaFormat::Yaml, selection, sort_order).unwrap();
            assert_eq!(expected, produced);
        }
    }
}
