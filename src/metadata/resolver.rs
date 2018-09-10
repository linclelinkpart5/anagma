use std::path::Path;
use std::path::PathBuf;
use std::collections::HashMap;
use std::borrow::Cow;

use failure::Error;

use metadata::types::MetaBlock;
use metadata::target::MetaTarget;
use metadata::reader::MetaReader;

pub fn get_simple_metadata<MR, P, MTS>(
    target_item_path: P,
    meta_targets: MTS,
    opt_cache: Option<&mut HashMap<PathBuf, MetaBlock>>,
) -> Result<Cow<MetaBlock>, Error>
where
    MR: MetaReader,
    P: AsRef<Path>,
    MTS: IntoIterator<Item = MetaTarget>,
{
    let target_item_path = target_item_path.as_ref();

    // If we have a cache, check to see if the requested path already has metadata.
    if let Some(cache) = opt_cache {
        if let Some(meta_block) = cache.get(target_item_path) {
            // Return the cached entry.
            return Ok(Cow::Borrowed(meta_block))
        }
    }

    let mut merged_metadata = MetaBlock::new();

    // If there is no cache provided, or if not found in the cache, read the metadata.
    // Use the provided meta targets.
    for meta_target in meta_targets.into_iter() {
        let target_meta_path = meta_target.get_target_meta_path(target_item_path)?;

        let metadata = MR::from_file(&target_meta_path, &meta_target)?;
    }
    // let self_mb =

    Ok(Cow::Owned(MetaBlock::new()))
}
