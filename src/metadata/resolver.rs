use std::path::Path;
use std::path::PathBuf;
use std::collections::HashMap;
use std::borrow::Cow;

use failure::Error;

use metadata::types::MetaBlock;
use metadata::location::MetaLocation;
use metadata::reader::MetaReader;
use metadata::plexer::MetaPlexer;

pub struct MetaResolver;

impl MetaResolver {
    pub fn process_meta_file<MR, P>(
        meta_path: P,
        meta_location: MetaLocation,
    ) -> Result<HashMap<PathBuf, HashMap<PathBuf, MetaBlock>>, Error>
    where
        MR: MetaReader,
        P: AsRef<Path>,
    {
        let meta_structure = MR::from_file(&meta_path, meta_location)?;

        let item_paths = meta_location.get_item_paths(&meta_path)?;

        let meta_plexed = MetaPlexer::plex(meta_structure, item_paths);

        Ok(hashmap![])
    }

    pub fn process_meta_file_cached<MR, P>(
        meta_path: P,
        meta_location: MetaLocation,
        cache: &mut HashMap<PathBuf, HashMap<PathBuf, MetaBlock>>,
        force: bool,
    ) -> Result<bool, Error>
    where
        MR: MetaReader,
        P: AsRef<Path>,
    {
        let meta_path = meta_path.as_ref();

        // Check to see if the requested meta path is already cached.
        if let Some(meta_block) = cache.get(meta_path) {
            // Return the cached entry.
            return Ok(true)
        }

        Ok(true)
    }

    pub fn get_simple_metadata<MR, P, MLS>(
        target_item_path: P,
        meta_locations: MLS,
        opt_cache: Option<&mut HashMap<PathBuf, MetaBlock>>,
    ) -> Result<Cow<MetaBlock>, Error>
    where
        MR: MetaReader,
        P: AsRef<Path>,
        MLS: IntoIterator<Item = MetaLocation>,
    {
        let target_item_path = target_item_path.as_ref();

        // If we have a cache, check to see if the requested path already has metadata.
        if let Some(cache) = opt_cache {
            if let Some(meta_block) = cache.get(target_item_path) {
                // Return the cached entry.
                return Ok(Cow::Borrowed(meta_block))
            }
        }

        let mut merged_meta_block = MetaBlock::new();

        // If there is no cache provided, or if not found in the cache, read the metadata.
        // Use the provided meta targets.
        for meta_location in meta_locations.into_iter() {
            let target_meta_path = meta_location.get_meta_path(target_item_path)?;

            let meta_structure = MR::from_file(&target_meta_path, meta_location)?;

            // TODO: Need a way to get "path extents": the set of item files governed by this meta file/location.

            // let possible_owned_item_paths = meta_location.get_possible_owned_item_paths

            // let plex_map = MetaPlexer::plex(meta_structure, item_paths)
        }

        Ok(Cow::Owned(MetaBlock::new()))
    }
}
