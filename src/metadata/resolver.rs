use std::path::Path;
use std::path::PathBuf;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

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
    ) -> Result<HashMap<PathBuf, MetaBlock>, Error>
    where
        MR: MetaReader,
        P: AsRef<Path>,
    {
        let meta_structure = MR::from_file(&meta_path, meta_location)?;

        let item_paths = meta_location.get_item_paths(&meta_path)?;

        let meta_plexed = MetaPlexer::plex(meta_structure, item_paths);

        Ok(meta_plexed)
    }

    pub fn process_meta_file_cached<MR, P>(
        meta_path: P,
        meta_location: MetaLocation,
        cache: &mut HashMap<PathBuf, HashMap<PathBuf, MetaBlock>>,
        force: bool,
    ) -> Result<&HashMap<PathBuf, MetaBlock>, Error>
    where
        MR: MetaReader,
        P: AsRef<Path>,
    {
        let meta_path = meta_path.as_ref();

        if force {
            cache.remove(meta_path);
        }

        let file = match cache.entry(meta_path.to_owned()) {
            Entry::Occupied(e) => e.into_mut(),
            Entry::Vacant(e) => e.insert(Self::process_meta_file::<MR, _>(meta_path, meta_location)?),
        };

        Ok(file)
    }

    pub fn process_item_file_cached<MR, P>(
        item_path: P,
        meta_location: MetaLocation,
        cache: &mut HashMap<PathBuf, HashMap<PathBuf, MetaBlock>>,
        force: bool,
    ) -> Result<&MetaBlock, Error>
    where
        MR: MetaReader,
        P: AsRef<Path>,
    {
        let meta_path = item_path.as_ref();
        let processed_meta_file = Self::process_meta_file_cached::<MR, _>(&meta_path, meta_location, cache, force)?;
        processed_meta_file.get(item_path.as_ref()).ok_or(bail!("item path not found in processed metadata: \"{}\"", item_path.as_ref().to_string_lossy()))
    }
}
