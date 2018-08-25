#![feature(generators)]
#![feature(generator_trait)]
#![feature(extern_prelude)]

#[macro_use] extern crate failure;
extern crate regex;
#[macro_use] extern crate maplit;
extern crate yaml_rust;
extern crate serde_yaml;
#[macro_use] extern crate serde_derive;
extern crate serde_regex;
extern crate globset;

mod library;
mod metadata;
mod util;

#[cfg(test)]
mod tests {
}
