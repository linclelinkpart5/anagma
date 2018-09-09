use std::path::Path;
use std::path::PathBuf;
use std::collections::HashMap;
use std::borrow::Cow;

use failure::Error;

use metadata::types::MetaBlock;
use metadata::reader::MetaReader;

pub fn get_simple_metadata<MR, P>(
    target_item_path: P,
    opt_cache: Option<&mut HashMap<PathBuf, MetaBlock>>,
) -> Result<Cow<MetaBlock>, Error>
where
    MR: MetaReader,
    P: AsRef<Path>,
{
    let target_item_path = target_item_path.as_ref();

    // If we have a cache, check to see if the requested path already has metadata.
    if let Some(cache) = opt_cache {
        if let Some(meta_block) = cache.get(target_item_path) {
            // Return the cached entry.
            return Ok(Cow::Borrowed(meta_block))
        }
    }

    Ok(Cow::Owned(MetaBlock::new()))
}
