//! High-level methods for processing meta files and loading item file metadata.

use std::path::Path;
use std::collections::HashMap;
use std::borrow::Cow;

use strum::IntoEnumIterator;

use crate::config::selection::Selection;
use crate::config::sorter::Sorter;
use crate::metadata::schema::Schema;
use crate::metadata::schema::SchemaFormat;
use crate::metadata::schema::Error as SchemaError;
use crate::metadata::block::Block;
use crate::metadata::target::Target;
use crate::metadata::target::Error as TargetError;
use crate::metadata::plexer::Plexer;
use crate::metadata::plexer::Error as PlexerError;

#[derive(Debug)]
pub enum Error {
    CannotReadMetadata(SchemaError),
    CannotFindItemPaths(TargetError),
    CannotFindMetaPath(TargetError),
    PlexerError(PlexerError),
    MissingMetadata,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::CannotReadMetadata(ref err) => write!(f, "cannot read metadata file: {}", err),
            Self::CannotFindItemPaths(ref err) => write!(f, "cannot find item file paths: {}", err),
            Self::CannotFindMetaPath(ref err) => write!(f, "cannot find meta file path: {}", err),
            Self::PlexerError(ref err) => write!(f, "plexing error: {}", err),
            Self::MissingMetadata => write!(f, "missing metadata"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::CannotReadMetadata(ref err) => Some(err),
            Self::CannotFindItemPaths(ref err) => Some(err),
            Self::CannotFindMetaPath(ref err) => Some(err),
            Self::PlexerError(ref err) => Some(err),
            Self::MissingMetadata => None,
        }
    }
}

pub struct Processor;

impl Processor {
    /// Processes the metadata contained in a target meta file.
    /// This loads and plexes metadata, and produces a mapping of item file
    /// paths to metadata blocks.
    pub fn process_meta_file<'a, P>(
        meta_path: &'a P,
        meta_target: Target,
        schema_format: SchemaFormat,
        selection: &Selection,
        sorter: Sorter,
    ) -> Result<HashMap<Cow<'a, Path>, Block>, Error>
    where
        P: AsRef<Path>,
    {
        // LEARN: Since `meta_path` is already a ref, no need to add `&`!
        let meta_structure =
            Schema::from_file(schema_format, meta_path, meta_target)
            .map_err(Error::CannotReadMetadata)?
        ;

        // LEARN: Since `meta_path` is already a ref, no need to add `&`!
        let selected_item_paths =
            meta_target
            .selected_item_paths(meta_path, selection)
            .map_err(Error::CannotFindItemPaths)?
        ;

        let mut meta_plexed = HashMap::new();

        let meta_plexer = Plexer::new(
            meta_structure,
            selected_item_paths.into_iter(),
            sorter,
        );

        for meta_plex_res in meta_plexer {
            let (item_path, meta_block) = meta_plex_res.map_err(Error::PlexerError)?;
            meta_plexed.insert(item_path, meta_block);
        }

        Ok(meta_plexed)
    }

    /// Processes metadata for a target item file.
    /// This performs the necessary merging of all metadata across different
    /// targets that may provide data for this item file. Merging is done in a
    /// "combine-last" fashion; if a later target produces the same metadata key
    /// as an earlier target, the later one wins and overwrites the earlier one.
    pub fn process_item_file<P>(
        item_path: &P,
        schema_format: SchemaFormat,
        selection: &Selection,
        sorter: Sorter,
    ) -> Result<Block, Error>
    where
        P: AsRef<Path>,
    {
        let mut comp_mb = Block::new();

        for meta_target in Target::iter() {
            let meta_path = match meta_target.meta_path(item_path, schema_format) {
                Err(e) => {
                    if e.is_fatal() { return Err(e).map_err(Error::CannotFindMetaPath); }
                    else { continue; }
                },
                Ok(p) => p,
            };

            let mut processed_meta_file = Self::process_meta_file(
                &meta_path,
                meta_target,
                schema_format,
                selection,
                sorter,
            )?;

            // The results of processing a meta file will often return extra
            // metadata for item files besides the targeted one. Extract the
            // target item file's metadata, and drop the remaining results.
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
    use super::*;

    use maplit::{hashmap, btreemap};

    use crate::test_util::create_temp_media_test_dir;

    use crate::test_util::TestUtil as TU;

    #[test]
    fn process_meta_file() {
        let temp_dir = create_temp_media_test_dir("test_process_meta_file");
        let path = temp_dir.path();

        let selection = Selection::default();
        let sorter = Sorter::default();

        // Success cases
        let inputs_and_expected = vec![
            // (
            //     (path.join("self.yml"), Target::Parent),
            //     hashmap![
            //         path.to_owned() => btreemap![
            //             "ROOT_self_key".to_owned() => TU::s("ROOT_self_val"),
            //             "const_key".to_owned() => TU::s("const_val"),
            //             "self_key".to_owned() => TU::s("self_val"),
            //             "overridden".to_owned() => TU::s("ROOT_self"),
            //         ],
            //     ],
            // ),
            (
                (path.join("item.yml"), Target::Siblings),
                hashmap![
                    Cow::Owned(path.join("ALBUM_01")) => btreemap![
                        String::from("ALBUM_01_item_key") => TU::s("ALBUM_01_item_val"),
                        String::from("const_key") => TU::s("const_val"),
                        String::from("item_key") => TU::s("item_val"),
                        String::from("overridden") => TU::s("ALBUM_01_item"),
                    ],
                    Cow::Owned(path.join("ALBUM_02")) => btreemap![
                        String::from("ALBUM_02_item_key") => TU::s("ALBUM_02_item_val"),
                        String::from("const_key") => TU::s("const_val"),
                        String::from("item_key") => TU::s("item_val"),
                        String::from("overridden") => TU::s("ALBUM_02_item"),
                    ],
                    Cow::Owned(path.join("ALBUM_03")) => btreemap![
                        String::from("ALBUM_03_item_key") => TU::s("ALBUM_03_item_val"),
                        String::from("const_key") => TU::s("const_val"),
                        String::from("item_key") => TU::s("item_val"),
                        String::from("overridden") => TU::s("ALBUM_03_item"),
                    ],
                    Cow::Owned(path.join("ALBUM_04.flac")) => btreemap![
                        String::from("ALBUM_04_item_key") => TU::s("ALBUM_04_item_val"),
                        String::from("const_key") => TU::s("const_val"),
                        String::from("item_key") => TU::s("item_val"),
                        String::from("overridden") => TU::s("ALBUM_04_item"),
                    ],
                    Cow::Owned(path.join("ALBUM_05")) => btreemap![
                        String::from("ALBUM_05_item_key") => TU::s("ALBUM_05_item_val"),
                        String::from("const_key") => TU::s("const_val"),
                        String::from("item_key") => TU::s("item_val"),
                        String::from("overridden") => TU::s("ALBUM_05_item"),
                    ],
                ],
            ),
            // (
            //     (path.join("ALBUM_01").join("self.yml"), Target::Parent),
            //     hashmap![
            //         path.join("ALBUM_01") => btreemap![
            //             "ALBUM_01_self_key".to_owned() => TU::s("ALBUM_01_self_val"),
            //             "const_key".to_owned() => TU::s("const_val"),
            //             "self_key".to_owned() => TU::s("self_val"),
            //             "overridden".to_owned() => TU::s("ALBUM_01_self"),
            //         ],
            //     ],
            // ),
            // (
            //     (path.join("ALBUM_01").join("DISC_01").join("item.yml"), Target::Siblings),
            //     hashmap![
            //         path.join("ALBUM_01").join("DISC_01").join("TRACK_01.flac") => btreemap![
            //             "TRACK_01_item_key".to_owned() => TU::s("TRACK_01_item_val"),
            //             "const_key".to_owned() => TU::s("const_val"),
            //             "item_key".to_owned() => TU::s("item_val"),
            //             "overridden".to_owned() => TU::s("TRACK_01_item"),
            //         ],
            //         path.join("ALBUM_01").join("DISC_01").join("TRACK_02.flac") => btreemap![
            //             "TRACK_02_item_key".to_owned() => TU::s("TRACK_02_item_val"),
            //             "const_key".to_owned() => TU::s("const_val"),
            //             "item_key".to_owned() => TU::s("item_val"),
            //             "overridden".to_owned() => TU::s("TRACK_02_item"),
            //         ],
            //         path.join("ALBUM_01").join("DISC_01").join("TRACK_03.flac") => btreemap![
            //             "TRACK_03_item_key".to_owned() => TU::s("TRACK_03_item_val"),
            //             "const_key".to_owned() => TU::s("const_val"),
            //             "item_key".to_owned() => TU::s("item_val"),
            //             "overridden".to_owned() => TU::s("TRACK_03_item"),
            //         ],
            //     ],
            // ),
        ];

        for (input, expected) in inputs_and_expected {
            let (meta_path, meta_target) = input;

            let produced = Processor::process_meta_file(&meta_path, meta_target, SchemaFormat::Yaml, &selection, sorter).unwrap();
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn process_item_file() {
        let temp_dir = create_temp_media_test_dir("test_process_item_file");
        let path = temp_dir.path();

        let selection = Selection::default();
        let sorter = Sorter::default();

        // Success cases
        let inputs_and_expected = vec![
            (
                Cow::Borrowed(path),
                btreemap![
                    String::from("ROOT_self_key") => TU::s("ROOT_self_val"),
                    String::from("const_key") => TU::s("const_val"),
                    String::from("self_key") => TU::s("self_val"),
                    String::from("overridden") => TU::s("ROOT_self"),
                ],
            ),
            (
                Cow::Owned(path.join("ALBUM_01")),
                btreemap![
                    String::from("ALBUM_01_item_key") => TU::s("ALBUM_01_item_val"),
                    String::from("ALBUM_01_self_key") => TU::s("ALBUM_01_self_val"),
                    String::from("const_key") => TU::s("const_val"),
                    String::from("item_key") => TU::s("item_val"),
                    String::from("self_key") => TU::s("self_val"),
                    String::from("overridden") => TU::s("ALBUM_01_self"),
                ],
            ),
            (
                Cow::Owned(path.join("ALBUM_01").join("DISC_01").join("TRACK_01.flac")),
                btreemap![
                    String::from("TRACK_01_item_key") => TU::s("TRACK_01_item_val"),
                    String::from("const_key") => TU::s("const_val"),
                    String::from("item_key") => TU::s("item_val"),
                    String::from("overridden") => TU::s("TRACK_01_item"),
                ],
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let item_path = input;

            let produced = Processor::process_item_file(&item_path, SchemaFormat::Yaml, &selection, sorter).unwrap();
            assert_eq!(expected, produced);
        }
    }
}
