pub mod config;
pub mod metadata;
pub mod source;
mod util;

#[cfg(test)] mod test_util;

use std::path::Path;

use crate::config::Config;
use crate::metadata::block::Block;
use crate::metadata::processor::Processor;
use crate::source::{Source, Sourcer, Anchor};

pub fn get<P: AsRef<Path>>(path: &P) -> Block {
    let config = Config::default();
    get_with_config(path, &config)
}

pub fn get_with_config<P: AsRef<Path>>(path: &P, config: &Config) -> Block {
    // TODO: Replace this with a `Sourcer` that comes from `Config`.
    let mut temp_sourcer = Sourcer::new();
    temp_sourcer.source(Source::from_name(String::from("item.json"), Anchor::External).unwrap())
        .source(Source::from_name(String::from("self.json"), Anchor::Internal).unwrap());

    Processor::process_item_file(
        path.as_ref(),
        &temp_sourcer,
        &config.schema_format,
        &config.selection,
        &config.sorter,
    ).unwrap()
}
