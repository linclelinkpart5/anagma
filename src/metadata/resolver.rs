//! Manages field-based lookups of metadata.

use std::path::Path;

use failure::Error;

use library::config::Config;
use metadata::types::MetaVal;
use metadata::types::MetaBlock;
use metadata::processor::MetaProcessor;
use metadata::reader::yaml::YamlMetaReader;
use metadata::location::MetaLocation;

const LOCATION_LIST: &[MetaLocation] = &[MetaLocation::Siblings, MetaLocation::Contains];

pub struct MetaResolver;

impl MetaResolver {
    pub fn resolve_field<P>(
        item_path: P,
        config: &Config,
    ) -> Result<MetaBlock, Error>
    where
        P: AsRef<Path>,
    {
        MetaProcessor::composite_item_file::<YamlMetaReader, _, _>(
            item_path,
            LOCATION_LIST.to_vec(),
            &config,
        )
    }
}
