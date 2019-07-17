#![feature(generators)]
#![feature(generator_trait)]
#![feature(non_exhaustive)]

#[macro_use] extern crate maplit;
extern crate serde;
extern crate serde_yaml;
extern crate serde_json;
#[macro_use] extern crate serde_derive;
extern crate globset;
extern crate itertools;
#[macro_use] extern crate log;
extern crate rust_decimal;
#[macro_use] extern crate rust_decimal_macros;
extern crate strum;
#[macro_use] extern crate strum_macros;

#[cfg(test)] extern crate tempfile;

pub mod metadata;
pub mod config;
mod util;
pub mod scripting;

#[cfg(test)] mod test_util;
