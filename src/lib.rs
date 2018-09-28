#![feature(generators)]
#![feature(generator_trait)]

#[macro_use] extern crate failure;
extern crate regex;
#[macro_use] extern crate maplit;
extern crate yaml_rust;
extern crate serde;
extern crate serde_yaml;
#[macro_use] extern crate serde_derive;
extern crate globset;
#[macro_use] extern crate log;
extern crate itertools;

#[cfg(test)] extern crate tempdir;

mod library;
mod metadata;
mod util;

#[cfg(test)] mod test_util;
