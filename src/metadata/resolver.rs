//! Manages field-based lookups of metadata.

use std::path::Path;
use std::marker::PhantomData;

use failure::Error;

use library::config::Config;
use metadata::types::MetaVal;
use metadata::processor::MetaProcessor;
use metadata::reader::MetaReader;
use metadata::location::MetaLocation;

const LOCATION_LIST: &[MetaLocation] = &[MetaLocation::Siblings, MetaLocation::Contains];

pub struct MetaResolver<MR>(PhantomData<MR>);

impl<MR> MetaResolver<MR>
where
    MR: MetaReader,
{
    pub fn resolve_field<P, S>(
        item_path: P,
        field: S,
        config: &Config,
    ) -> Result<Option<MetaVal>, Error>
    where
        P: AsRef<Path>,
        S: AsRef<str>,
    {
        let mut mb = MetaProcessor::<MR>::composite_item_file(
            item_path,
            LOCATION_LIST.to_vec(),
            &config,
        )?;

        Ok(mb.remove(field.as_ref()))
    }

    pub fn resolve_field_parents<P, S>(
        item_path: P,
        field: S,
        config: &Config,
    ) -> Result<Option<MetaVal>, Error>
    where
        P: AsRef<Path>,
        S: AsRef<str>,
    {
        // LEARN: The first item in `.ancestors()` is the original path, so it needs to be skipped.
        for ancestor_item_path in item_path.as_ref().ancestors().into_iter().skip(1) {
            let opt_val = Self::resolve_field(&item_path, &field, &config)?;

            if let Some(val) = opt_val {
                return Ok(Some(val))
            }
        }

        Ok(None)
    }
}
