//! This is intended to be the main public API of the library.

use std::path::Path;
use std::path::PathBuf;
use std::collections::BTreeMap;

use library::config::Config;
use metadata::types::MetaBlock;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum AggMethod {
    Collect,
    First,
}

pub struct MetaFinalizer;

impl MetaFinalizer {
    pub fn get_metadata<P: AsRef<Path>>(item_path: P) -> Result<MetaBlock, ()> {
        // Use a default configuration and no aggregations.
        let config = Config::default();
        let agg_methods = BTreeMap::new();

        Self::get_metadata_with_config_and_aggs(item_path, &config, &agg_methods)
    }

    pub fn get_metadata_with_config<P: AsRef<Path>>(item_path: P, config: &Config) -> Result<MetaBlock, ()> {
        let agg_methods = BTreeMap::new();

        Self::get_metadata_with_config_and_aggs(item_path, config, &agg_methods)
    }

    pub fn get_metadata_with_aggs<P: AsRef<Path>>(item_path: P, agg_methods: &BTreeMap<String, AggMethod>) -> Result<MetaBlock, ()> {
        let config = Config::default();

        Self::get_metadata_with_config_and_aggs(item_path, &config, agg_methods)
    }

    pub fn get_metadata_with_config_and_aggs<P: AsRef<Path>>(
        item_path: P,
        config: &Config,
        agg_methods: &BTreeMap<String, AggMethod>,
    ) -> Result<MetaBlock, ()>
    {
        Ok(MetaBlock::new())
    }
}
