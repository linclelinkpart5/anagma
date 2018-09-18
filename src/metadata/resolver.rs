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
            let target_meta_path = meta_location.get_owning_meta_path(target_item_path)?;

            let meta_structure = MR::from_file(&target_meta_path, &meta_location)?;

            // TODO: Need a way to get "path extents": the set of item files governed by this meta file/location.

            // let plex_map = MetaPlexer::plex(meta_structure, item_paths)
        }

        Ok(Cow::Owned(MetaBlock::new()))
    }
}
