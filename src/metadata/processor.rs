use std::path::Path;
use std::path::PathBuf;
use std::collections::HashMap;
use std::marker::PhantomData;

use library::selection::Selection;
use library::sort_order::SortOrder;
use metadata::types::MetaBlock;
use metadata::location::MetaLocation;
use metadata::location::Error as LocationError;
use metadata::reader::MetaReader;
use metadata::reader::Error as ReaderError;
use metadata::plexer::MetaPlexer;

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

pub struct MetaProcessor<MR>(PhantomData<MR>);

impl<MR> MetaProcessor<MR>
where
    MR: MetaReader,
{
    pub fn process_meta_file<P>(
        meta_path: P,
        meta_location: MetaLocation,
        selection: &Selection,
        sort_order: SortOrder,
    ) -> Result<HashMap<PathBuf, MetaBlock>, Error>
    where
        P: AsRef<Path>,
    {
        let meta_structure = MR::from_file(&meta_path, meta_location).map_err(Error::CannotReadMetadata)?;

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

    pub fn process_item_file<P>(
        item_path: P,
        meta_location: MetaLocation,
        selection: &Selection,
        sort_order: SortOrder,
    ) -> Result<MetaBlock, Error>
    where
        P: AsRef<Path>,
    {
        let meta_path = match meta_location.get_meta_path(&item_path) {
            Err(e) => {
                match e {
                    LocationError::NonexistentMetaPath(..) |
                        LocationError::InvalidItemDirPath(..) |
                        LocationError::NoItemPathParent(..) => { return Ok(MetaBlock::new()); },
                    _ => { return Err(e).map_err(Error::CannotFindMetaPath)?; }
                }
            },
            Ok(p) => p,
        };

        // let meta_path = meta_location.get_meta_path(&item_path).context(Error::CannotFindMetaPath)?;

        let mut processed_meta_file = Self::process_meta_file(&meta_path, meta_location, selection, sort_order)?;

        // The remaining results can be thrown away.
        if let Some(meta_block) = processed_meta_file.remove(item_path.as_ref()) {
            Ok(meta_block)
        }
        else {
            Err(Error::MissingMetadata)?
        }
    }

    // Processes multiple locations for a target item at once, merging the results.
    // Merging is "combine-last", so matching result keys for subsequent locations override earlier keys.
    pub fn process_item_file_flattened<P, II>(
        item_path: P,
        meta_locations: II,
        selection: &Selection,
        sort_order: SortOrder,
    ) -> Result<MetaBlock, Error>
    where
        P: AsRef<Path>,
        II: IntoIterator<Item = MetaLocation>,
    {
        let mut comp_mb = MetaBlock::new();

        for meta_location in meta_locations.into_iter() {
            comp_mb.extend(Self::process_item_file(&item_path, meta_location, selection, sort_order)?);
        }

        Ok(comp_mb)
    }
}

#[cfg(test)]
mod tests {
    use super::MetaProcessor;

    use library::config::Config;
    use metadata::reader::yaml::YamlMetaReader;
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

            let produced = MetaProcessor::<YamlMetaReader>::process_meta_file(meta_path, meta_location, selection, sort_order).unwrap();
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
                (path.to_owned(), MetaLocation::Contains),
                btreemap![
                    "ROOT_self_key".to_owned() => MetaVal::Str("ROOT_self_val".to_owned()),
                    "const_key".to_owned() => MetaVal::Str("const_val".to_owned()),
                    "self_key".to_owned() => MetaVal::Str("self_val".to_owned()),
                    "overridden".to_owned() => MetaVal::Str("ROOT_self".to_owned()),
                ],
            ),
            (
                (path.join("ALBUM_01"), MetaLocation::Siblings),
                btreemap![
                    "ALBUM_01_item_key".to_owned() => MetaVal::Str("ALBUM_01_item_val".to_owned()),
                    "const_key".to_owned() => MetaVal::Str("const_val".to_owned()),
                    "item_key".to_owned() => MetaVal::Str("item_val".to_owned()),
                    "overridden".to_owned() => MetaVal::Str("ALBUM_01_item".to_owned()),
                ],
            ),
            (
                (path.join("ALBUM_01"), MetaLocation::Contains),
                btreemap![
                    "ALBUM_01_self_key".to_owned() => MetaVal::Str("ALBUM_01_self_val".to_owned()),
                    "const_key".to_owned() => MetaVal::Str("const_val".to_owned()),
                    "self_key".to_owned() => MetaVal::Str("self_val".to_owned()),
                    "overridden".to_owned() => MetaVal::Str("ALBUM_01_self".to_owned()),
                ],
            ),
            (
                (path.join("ALBUM_01").join("DISC_01").join("TRACK_01.flac"), MetaLocation::Siblings),
                btreemap![
                    "TRACK_01_item_key".to_owned() => MetaVal::Str("TRACK_01_item_val".to_owned()),
                    "const_key".to_owned() => MetaVal::Str("const_val".to_owned()),
                    "item_key".to_owned() => MetaVal::Str("item_val".to_owned()),
                    "overridden".to_owned() => MetaVal::Str("TRACK_01_item".to_owned()),
                ],
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let (item_path, meta_location) = input;

            let produced = MetaProcessor::<YamlMetaReader>::process_item_file(item_path, meta_location, selection, sort_order).unwrap();
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_process_item_file_flattened() {
        let temp_dir = create_temp_media_test_dir("test_process_item_file_flattened");
        let path = temp_dir.path();

        // use std::thread::sleep_ms;
        // sleep_ms(10000000);

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

        let meta_locations = vec![MetaLocation::Siblings, MetaLocation::Contains];

        for (input, expected) in inputs_and_expected {
            let item_path = input;

            let produced = MetaProcessor::<YamlMetaReader>::process_item_file_flattened(item_path, meta_locations.clone(), selection, sort_order).unwrap();
            assert_eq!(expected, produced);
        }
    }
}
