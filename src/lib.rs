#![feature(generators)]
#![feature(generator_trait)]

#[macro_use] extern crate failure;
extern crate regex;
#[macro_use] extern crate maplit;

mod library;
mod metadata;
mod util;

#[cfg(test)]
mod tests {
}
