pub mod config;
pub mod metadata;
pub mod source;
mod util;

#[cfg(test)] mod test_util;

use std::path::Path;

use crate::config::Config;
use crate::metadata::block::Block;
use crate::metadata::processor::Processor;

pub fn get<P: AsRef<Path>>(path: &P) -> Block {
    let config = Config::default();
    get_with_config(path, &config)
}

pub fn get_with_config<P: AsRef<Path>>(path: &P, config: &Config) -> Block {
    Processor::process_item_file(
        path.as_ref(),
        &config.sourcer,
        &config.selection,
        &config.sorter,
    ).unwrap()
}
