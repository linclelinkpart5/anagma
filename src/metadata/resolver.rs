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
    ) -> Result<MetaVal, Error>
    where
        P: AsRef<Path>,
        S: AsRef<str>,
    {
        let mut mb = MetaProcessor::<MR>::composite_item_file(
            item_path,
            LOCATION_LIST.to_vec(),
            &config,
        )?;

        match mb.remove(field.as_ref()) {
            Some(val) => Ok(val),
            None => Ok(MetaVal::Nil),
        }
    }
}
