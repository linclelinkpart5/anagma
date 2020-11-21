//! High-level methods for processing meta files and loading item file metadata.

use std::borrow::Cow;
use std::collections::HashMap;
use std::path::Path;

use thiserror::Error;

use crate::config::selection::Selection;
use crate::config::sorter::Sorter;
use crate::metadata::block::Block;
use crate::metadata::plexer::Error as PlexerError;
use crate::metadata::plexer::Plexer;
use crate::metadata::schema::Error as SchemaError;
use crate::metadata::schema::Schema;
use crate::metadata::schema::SchemaFormat;
use crate::source::{Error as SourceError, Source, Sourcer};

#[derive(Debug, Error)]
pub enum Error {
    #[error("cannot read metadata file: {0}")]
    CannotReadMetadata(#[source] SchemaError),
    #[error("cannot find item file paths: {0}")]
    CannotFindItemPaths(#[source] SourceError),
    #[error("cannot find meta file path: {0}")]
    CannotFindMetaPath(#[source] SourceError),
    #[error("plexing error: {0}")]
    PlexerError(#[source] PlexerError),
    #[error("missing metadata")]
    MissingMetadata,
}

pub struct Processor;

impl Processor {
    /// Processes the metadata contained in a target meta file.
    /// This loads and plexes metadata, and produces a mapping of item file
    /// paths to metadata blocks.
    pub fn process_meta_file<'a>(
        meta_path: &'a Path,
        source: &'a Source,
        selection: &'a Selection,
        sorter: &'a Sorter,
    ) -> Result<HashMap<Cow<'a, Path>, Block>, Error> {
        let arity = source.anchor.into();
        let schema = Schema::from_file(&source.format, meta_path, &arity)
            .map_err(Error::CannotReadMetadata)?;

        // LEARN: Since `meta_path` is already a ref, no need to add `&`!
        let sel_item_paths = source
            .selected_item_paths(meta_path, selection)
            .map_err(Error::CannotFindItemPaths)?;

        // TODO: Temporary compatibility measure until plexer can accept an
        //       iterator.
        let mut sel_item_paths = sel_item_paths
            .collect::<Result<Vec<_>, _>>()
            .expect("TEMPORARY");

        if schema.expects_sorted() {
            // Sort the input item paths.
            sel_item_paths.sort_by(|a, b| sorter.path_sort_cmp(a, b));
        }

        let mut meta_plexed = HashMap::new();

        let meta_plexer = Plexer::new(schema, sel_item_paths.into_iter());

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
    pub fn process_item_file(
        item_path: &Path,
        sourcer: &Sourcer,
        selection: &Selection,
        sorter: &Sorter,
    ) -> Result<Block, Error> {
        let mut comp_mb = Block::new();

        let meta_paths = sourcer.meta_paths(item_path);

        for mps_res in meta_paths {
            let (meta_path, source) = mps_res.map_err(Error::CannotFindMetaPath)?;

            let mut processed_meta_file =
                Self::process_meta_file(&meta_path, source, selection, sorter)?;

            // The results of processing a meta file will often return extra
            // metadata for item files besides the targeted one. Extract the
            // target item file's metadata, and drop the remaining results.
            if let Some(meta_block) = processed_meta_file.remove(item_path) {
                comp_mb.extend(meta_block)
            } else {
                Err(Error::MissingMetadata)?
            }
        }

        Ok(comp_mb)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use maplit::{btreemap, hashmap};
    use str_macro::str;

    use crate::config::selection::Matcher;
    use crate::source::Anchor;
    use crate::test_util::TestUtil as TU;

    #[test]
    fn process_meta_file() {
        let temp_dir = TU::create_temp_media_test_dir("process_meta_file");
        let path = temp_dir.path();

        let selection = Selection::new(
            Matcher::any(),
            Matcher::build(&["*.json"]).unwrap(),
            Matcher::any(),
            Matcher::empty(),
        );
        let sorter = Sorter::default();

        // Success cases
        let inputs_and_expected = vec![
            (
                (
                    path.join("self.json"),
                    Source::from_name(str!("self.json"), Anchor::Internal).unwrap(),
                ),
                hashmap![
                    Cow::Owned(path.to_owned()) => btreemap![
                        str!("ROOT_self_key") => TU::s("ROOT_self_val"),
                        str!("const_key") => TU::s("const_val"),
                        str!("self_key") => TU::s("self_val"),
                        str!("overridden") => TU::s("ROOT_self"),
                    ],
                ],
            ),
            (
                (
                    path.join("item.json"),
                    Source::from_name(str!("item.json"), Anchor::External).unwrap(),
                ),
                hashmap![
                    Cow::Owned(path.join("ALBUM_01")) => btreemap![
                        str!("ALBUM_01_item_key") => TU::s("ALBUM_01_item_val"),
                        str!("const_key") => TU::s("const_val"),
                        str!("item_key") => TU::s("item_val"),
                        str!("overridden") => TU::s("ALBUM_01_item"),
                    ],
                    Cow::Owned(path.join("ALBUM_02")) => btreemap![
                        str!("ALBUM_02_item_key") => TU::s("ALBUM_02_item_val"),
                        str!("const_key") => TU::s("const_val"),
                        str!("item_key") => TU::s("item_val"),
                        str!("overridden") => TU::s("ALBUM_02_item"),
                    ],
                    Cow::Owned(path.join("ALBUM_03")) => btreemap![
                        str!("ALBUM_03_item_key") => TU::s("ALBUM_03_item_val"),
                        str!("const_key") => TU::s("const_val"),
                        str!("item_key") => TU::s("item_val"),
                        str!("overridden") => TU::s("ALBUM_03_item"),
                    ],
                    Cow::Owned(path.join("ALBUM_04.flac")) => btreemap![
                        str!("ALBUM_04_item_key") => TU::s("ALBUM_04_item_val"),
                        str!("const_key") => TU::s("const_val"),
                        str!("item_key") => TU::s("item_val"),
                        str!("overridden") => TU::s("ALBUM_04_item"),
                    ],
                    Cow::Owned(path.join("ALBUM_05")) => btreemap![
                        str!("ALBUM_05_item_key") => TU::s("ALBUM_05_item_val"),
                        str!("const_key") => TU::s("const_val"),
                        str!("item_key") => TU::s("item_val"),
                        str!("overridden") => TU::s("ALBUM_05_item"),
                    ],
                ],
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let (meta_path, source) = input;

            let produced = Processor::process_meta_file(
                &meta_path,
                &source,
                &selection,
                &sorter,
            )
            .unwrap();
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn process_item_file() {
        let temp_dir = TU::create_temp_media_test_dir("process_item_file");
        let path = temp_dir.path();

        let selection = Selection::new(
            Matcher::any(),
            Matcher::build(&["*.json"]).unwrap(),
            Matcher::any(),
            Matcher::empty(),
        );
        let sorter = Sorter::default();
        let mut sourcer = Sourcer::new();
        sourcer
            .source(Source::from_name(str!("item.json"), Anchor::External).unwrap())
            .source(Source::from_name(str!("self.json"), Anchor::Internal).unwrap());

        // Success cases
        let inputs_and_expected = vec![
            (
                Cow::Borrowed(path),
                btreemap![
                    str!("ROOT_self_key") => TU::s("ROOT_self_val"),
                    str!("const_key") => TU::s("const_val"),
                    str!("self_key") => TU::s("self_val"),
                    str!("overridden") => TU::s("ROOT_self"),
                ],
            ),
            (
                Cow::Owned(path.join("ALBUM_01")),
                btreemap![
                    str!("ALBUM_01_item_key") => TU::s("ALBUM_01_item_val"),
                    str!("ALBUM_01_self_key") => TU::s("ALBUM_01_self_val"),
                    str!("const_key") => TU::s("const_val"),
                    str!("item_key") => TU::s("item_val"),
                    str!("self_key") => TU::s("self_val"),
                    str!("overridden") => TU::s("ALBUM_01_self"),
                ],
            ),
            (
                Cow::Owned(path.join("ALBUM_01").join("DISC_01").join("TRACK_01.flac")),
                btreemap![
                    str!("TRACK_01_item_key") => TU::s("TRACK_01_item_val"),
                    str!("const_key") => TU::s("const_val"),
                    str!("item_key") => TU::s("item_val"),
                    str!("overridden") => TU::s("TRACK_01_item"),
                ],
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let item_path = input;

            let produced = Processor::process_item_file(
                &item_path,
                &sourcer,
                &selection,
                &sorter,
            )
            .unwrap();
            assert_eq!(expected, produced);
        }
    }
}
