use std::path::{Path, PathBuf};

use crate::sources::{Source, SourceError};

// Represents an ordered collection of `Source`s, designed to find meta files
// for a target item path.
#[derive(Debug)]
pub struct Sourcer(Vec<Source>);

impl Sourcer {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn source(&mut self, source: Source) -> &mut Self {
        self.0.push(source);
        self
    }

    pub fn meta_paths<'a>(&'a self, item_path: &'a Path) -> MetaPaths<'a> {
        MetaPaths {
            iter: self.0.iter(),
            item_path,
        }
    }

    pub fn as_sources(&self) -> &[Source] {
        self.0.as_slice()
    }
}

impl From<Vec<Source>> for Sourcer {
    fn from(value: Vec<Source>) -> Self {
        Self(value)
    }
}

pub struct MetaPaths<'a> {
    iter: std::slice::Iter<'a, Source>,
    item_path: &'a Path,
}

impl<'a> Iterator for MetaPaths<'a> {
    type Item = Result<(PathBuf, &'a Source), SourceError>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(source) = self.iter.next() {
            let res = source.meta_path(self.item_path);

            match res {
                Ok(meta_path) => {
                    return Some(Ok((meta_path, source)));
                }
                Err(err) if err.is_fatal() => {
                    return Some(Err(err));
                }
                Err(_) => {
                    continue;
                }
            }
        }

        None
    }
}
