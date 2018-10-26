use std::fmt::Display;
use std::fmt::Result as FmtResult;
use std::fmt::Formatter;

use failure::Backtrace;
use failure::Context;
use failure::Fail;
use failure::ResultExt;

#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Fail, Hash)]
#[non_exhaustive]
pub enum ErrorKind {
    #[fail(display = "cannot read metadata file")]
    CannotReadMetadata,
    #[fail(display = "cannot find item file paths")]
    CannotFindItemPaths,
    #[fail(display = "cannot find meta file path")]
    CannotFindMetaPath,
    #[fail(display = "missing metadata")]
    MissingMetadata,
}

impl Fail for Error {
    fn cause(&self) -> Option<&Fail> { self.inner.cause() }
    fn backtrace(&self) -> Option<&Backtrace> { self.inner.backtrace() }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult { Display::fmt(&self.inner, f) }
}

impl Error {
    pub fn kind(&self) -> &ErrorKind { self.inner.get_context() }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error { Error { inner: Context::new(kind) } }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Error { Error { inner: inner } }
}

use std::path::Path;
use std::path::PathBuf;
use std::collections::HashMap;
use std::marker::PhantomData;

use library::config::Config;
use metadata::types::MetaBlock;
use metadata::location::MetaLocation;
use metadata::location::ErrorKind as LocationErrorKind;
use metadata::reader::MetaReader;
use metadata::reader::ErrorKind as ReaderErrorKind;
use metadata::plexer::MetaPlexer;
use metadata::plexer::ErrorKind as PlexErrorKind;

pub struct MetaProcessor<MR>(PhantomData<MR>);

impl<MR> MetaProcessor<MR>
where
    MR: MetaReader,
{
    pub fn process_meta_file<P>(
        meta_path: P,
        meta_location: MetaLocation,
        config: &Config,
    ) -> Result<HashMap<PathBuf, MetaBlock>, Error>
    where
        P: AsRef<Path>,
    {
        let meta_structure = match MR::from_file(&meta_path, meta_location) {
            Err(e) => {
                let ek = e.kind().clone();
                match ek {
                    ReaderErrorKind::FileOpen => { return Ok(hashmap![]); },
                    _ => { return Err(e).context(ErrorKind::CannotReadMetadata)?; }
                }
            },
            Ok(ms) => ms,
        };

        // let meta_structure = MR::from_file(&meta_path, meta_location).context(ErrorKind::CannotReadMetadata)?;

        let selected_item_paths = meta_location.get_selected_item_paths(&meta_path, config).context(ErrorKind::CannotFindItemPaths)?;

        let mut meta_plexed = hashmap![];
        for meta_plex_res in MetaPlexer::plex(meta_structure, selected_item_paths, config.sort_order) {
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
        config: &Config,
    ) -> Result<MetaBlock, Error>
    where
        P: AsRef<Path>,
    {
        let meta_path = match meta_location.get_meta_path(&item_path) {
            Err(e) => {
                let ek = e.kind().clone();
                match ek {
                    LocationErrorKind::NonexistentMetaPath |
                        LocationErrorKind::InvalidItemDirPath |
                        LocationErrorKind::NoItemPathParent => { return Ok(MetaBlock::new()); },
                    _ => { return Err(e).context(ErrorKind::CannotFindMetaPath)?; }
                }
            },
            Ok(p) => p,
        };

        // let meta_path = meta_location.get_meta_path(&item_path).context(ErrorKind::CannotFindMetaPath)?;

        let mut processed_meta_file = Self::process_meta_file(&meta_path, meta_location, config)?;

        // The remaining results can be thrown away.
        if let Some(meta_block) = processed_meta_file.remove(item_path.as_ref()) {
            Ok(meta_block)
        }
        else {
            Err(ErrorKind::MissingMetadata)?
        }
    }

    // Processes multiple locations for a target item at once, merging the results.
    // Merging is "combine-last", so matching result keys for subsequent locations override earlier keys.
    pub fn process_item_file_flattened<P, II>(
        item_path: P,
        meta_locations: II,
        config: &Config,
    ) -> Result<MetaBlock, Error>
    where
        P: AsRef<Path>,
        II: IntoIterator<Item = MetaLocation>,
    {
        let mut comp_mb = MetaBlock::new();

        for meta_location in meta_locations.into_iter() {
            let mut mb = Self::process_item_file(&item_path, meta_location, &config)?;

            comp_mb.extend(mb);
        }

        Ok(comp_mb)
    }

    // pub fn process_meta_file_cached<'c, MR, P>(
    //     meta_path: P,
    //     meta_location: MetaLocation,
    //     config: &Config,
    //     cache: &'c mut HashMap<PathBuf, HashMap<PathBuf, MetaBlock>>,
    //     force: bool,
    // ) -> Result<&'c HashMap<PathBuf, MetaBlock>, Error>
    // where
    //     MR: MetaReader,
    //     P: AsRef<Path>,
    // {
    //     let meta_path = meta_path.as_ref();

    //     if force {
    //         cache.remove(meta_path);
    //     }

    //     let meta_file_results = match cache.entry(meta_path.to_owned()) {
    //         Entry::Occupied(e) => e.into_mut(),
    //         Entry::Vacant(e) => e.insert(Self::process_meta_file::<MR, _>(meta_path, meta_location, config)?),
    //     };

    //     Ok(meta_file_results)
    // }

    // pub fn process_item_file_cached<'c, MR, P>(
    //     item_path: P,
    //     meta_location: MetaLocation,
    //     config: &Config,
    //     cache: &'c mut HashMap<PathBuf, HashMap<PathBuf, MetaBlock>>,
    //     force: bool,
    // ) -> Result<&'c MetaBlock, Error>
    // where
    //     MR: MetaReader,
    //     P: AsRef<Path>,
    // {
    //     let meta_path = meta_location.get_meta_path(&item_path)?;

    //     let processed_meta_file = Self::process_meta_file_cached::<MR, _>(&meta_path, meta_location, config, cache, force)?;
    //     processed_meta_file.get(item_path.as_ref())
    //         .ok_or(bail!("item path not found in processed metadata: \"{}\"", item_path.as_ref().to_string_lossy()))
    // }
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

        // Success cases
        let inputs_and_expected = vec![
            (
                (path.join("self.yml"), MetaLocation::Contains),
                hashmap![
                    path.to_owned() => btreemap![
                        "ROOT_self_key".to_owned() => MetaVal::Str("ROOT_self_val".to_owned()),
                        "const_key".to_owned() => MetaVal::Str("const_val".to_owned()),
                        "self_key".to_owned() => MetaVal::Str("self_val".to_owned()),
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
                    ],
                    path.join("ALBUM_02") => btreemap![
                        "ALBUM_02_item_key".to_owned() => MetaVal::Str("ALBUM_02_item_val".to_owned()),
                        "const_key".to_owned() => MetaVal::Str("const_val".to_owned()),
                        "item_key".to_owned() => MetaVal::Str("item_val".to_owned()),
                    ],
                    path.join("ALBUM_03") => btreemap![
                        "ALBUM_03_item_key".to_owned() => MetaVal::Str("ALBUM_03_item_val".to_owned()),
                        "const_key".to_owned() => MetaVal::Str("const_val".to_owned()),
                        "item_key".to_owned() => MetaVal::Str("item_val".to_owned()),
                    ],
                    path.join("ALBUM_04.flac") => btreemap![
                        "ALBUM_04_item_key".to_owned() => MetaVal::Str("ALBUM_04_item_val".to_owned()),
                        "const_key".to_owned() => MetaVal::Str("const_val".to_owned()),
                        "item_key".to_owned() => MetaVal::Str("item_val".to_owned()),
                    ],
                    path.join("ALBUM_05") => btreemap![
                        "ALBUM_05_item_key".to_owned() => MetaVal::Str("ALBUM_05_item_val".to_owned()),
                        "const_key".to_owned() => MetaVal::Str("const_val".to_owned()),
                        "item_key".to_owned() => MetaVal::Str("item_val".to_owned()),
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
                    ],
                    path.join("ALBUM_01").join("DISC_01").join("TRACK_02.flac") => btreemap![
                        "TRACK_02_item_key".to_owned() => MetaVal::Str("TRACK_02_item_val".to_owned()),
                        "const_key".to_owned() => MetaVal::Str("const_val".to_owned()),
                        "item_key".to_owned() => MetaVal::Str("item_val".to_owned()),
                    ],
                    path.join("ALBUM_01").join("DISC_01").join("TRACK_03.flac") => btreemap![
                        "TRACK_03_item_key".to_owned() => MetaVal::Str("TRACK_03_item_val".to_owned()),
                        "const_key".to_owned() => MetaVal::Str("const_val".to_owned()),
                        "item_key".to_owned() => MetaVal::Str("item_val".to_owned()),
                    ],
                ],
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let (meta_path, meta_location) = input;

            let produced = MetaProcessor::<YamlMetaReader>::process_meta_file(meta_path, meta_location, &config).unwrap();
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_process_item_file() {
        let temp_dir = create_temp_media_test_dir("test_process_item_file");
        let path = temp_dir.path();

        let config = Config::default();

        // Success cases
        let inputs_and_expected = vec![
            (
                (path.to_owned(), MetaLocation::Contains),
                btreemap![
                    "ROOT_self_key".to_owned() => MetaVal::Str("ROOT_self_val".to_owned()),
                    "const_key".to_owned() => MetaVal::Str("const_val".to_owned()),
                    "self_key".to_owned() => MetaVal::Str("self_val".to_owned()),
                ],
            ),
            (
                (path.join("ALBUM_01"), MetaLocation::Siblings),
                btreemap![
                    "ALBUM_01_item_key".to_owned() => MetaVal::Str("ALBUM_01_item_val".to_owned()),
                    "const_key".to_owned() => MetaVal::Str("const_val".to_owned()),
                    "item_key".to_owned() => MetaVal::Str("item_val".to_owned()),
                ],
            ),
            (
                (path.join("ALBUM_01"), MetaLocation::Contains),
                btreemap![
                    "ALBUM_01_self_key".to_owned() => MetaVal::Str("ALBUM_01_self_val".to_owned()),
                    "const_key".to_owned() => MetaVal::Str("const_val".to_owned()),
                    "self_key".to_owned() => MetaVal::Str("self_val".to_owned()),
                ],
            ),
            (
                (path.join("ALBUM_01").join("DISC_01").join("TRACK_01.flac"), MetaLocation::Siblings),
                btreemap![
                    "TRACK_01_item_key".to_owned() => MetaVal::Str("TRACK_01_item_val".to_owned()),
                    "const_key".to_owned() => MetaVal::Str("const_val".to_owned()),
                    "item_key".to_owned() => MetaVal::Str("item_val".to_owned()),
                ],
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let (item_path, meta_location) = input;

            let produced = MetaProcessor::<YamlMetaReader>::process_item_file(item_path, meta_location, &config).unwrap();
            assert_eq!(expected, produced);
        }

        // let result = MetaProcessor::process_item_file::<YamlMetaReader, _>(path.join("ALBUM_01"), MetaLocation::Contains, &config);

        // println!("{:?}", result);
    }

    #[test]
    fn test_process_item_file_flattened() {
        let temp_dir = create_temp_media_test_dir("test_process_item_file_flattened");
        let path = temp_dir.path();

        let config = Config::default();

        // Success cases
        let inputs_and_expected = vec![
            (
                (path.to_owned(), MetaLocation::Contains),
                btreemap![
                    "ROOT_self_key".to_owned() => MetaVal::Str("ROOT_self_val".to_owned()),
                    "const_key".to_owned() => MetaVal::Str("const_val".to_owned()),
                    "self_key".to_owned() => MetaVal::Str("self_val".to_owned()),
                ],
            ),
            (
                (path.join("ALBUM_01"), MetaLocation::Siblings),
                btreemap![
                    "ALBUM_01_item_key".to_owned() => MetaVal::Str("ALBUM_01_item_val".to_owned()),
                    "const_key".to_owned() => MetaVal::Str("const_val".to_owned()),
                    "item_key".to_owned() => MetaVal::Str("item_val".to_owned()),
                ],
            ),
            (
                (path.join("ALBUM_01"), MetaLocation::Contains),
                btreemap![
                    "ALBUM_01_self_key".to_owned() => MetaVal::Str("ALBUM_01_self_val".to_owned()),
                    "const_key".to_owned() => MetaVal::Str("const_val".to_owned()),
                    "self_key".to_owned() => MetaVal::Str("self_val".to_owned()),
                ],
            ),
            (
                (path.join("ALBUM_01").join("DISC_01").join("TRACK_01.flac"), MetaLocation::Siblings),
                btreemap![
                    "TRACK_01_item_key".to_owned() => MetaVal::Str("TRACK_01_item_val".to_owned()),
                    "const_key".to_owned() => MetaVal::Str("const_val".to_owned()),
                    "item_key".to_owned() => MetaVal::Str("item_val".to_owned()),
                ],
            ),
        ];

        let meta_locations = vec![MetaLocation::Siblings, MetaLocation::Contains];

        for (input, expected) in inputs_and_expected {
            let (item_path, meta_location) = input;

            println!("{:?}", item_path);
            let result = MetaProcessor::<YamlMetaReader>::process_item_file_flattened(item_path, meta_locations.clone(), &config);
            eprintln!("{:?}", result);
            // let produced = MetaProcessor::<YamlMetaReader>::process_item_file_flattened(item_path, meta_locations.clone(), &config).unwrap();
            // assert_eq!(expected, produced);
        }

        // let result = MetaProcessor::process_item_file::<YamlMetaReader, _>(path.join("ALBUM_01"), MetaLocation::Contains, &config);

        // println!("{:?}", result);
    }
}
