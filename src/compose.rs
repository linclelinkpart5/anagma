use std::borrow::Cow;
use std::io::{Error as IoError, Result as IoResult, ErrorKind as IoErrorKind};
use std::path::{Path, PathBuf};

use crate::config::selection::Selection;
use crate::source::{Anchor, Source};

pub struct Compositor(Vec<Source>);

impl<'a> Compositor {
    pub(crate) fn new() -> Self {
        Self(Vec::new())
    }

    fn add_source<I>(&mut self, file_name: I, anchor: Anchor) -> &mut Self
    where
        I: Into<String>,
    {
        let file_name = file_name.into();

        let src = Source {
            file_name,
            anchor,
        };

        self.0.push(src);
        self
    }

    pub(crate) fn external<I>(&mut self, file_name: I) -> &mut Self
    where
        I: Into<String>,
    {
        self.add_source(file_name, Anchor::External)
    }

    pub(crate) fn internal<I>(&mut self, file_name: I) -> &mut Self
    where
        I: Into<String>,
    {
        self.add_source(file_name, Anchor::Internal)
    }

    pub fn compose(&self, item_path: &Path) {}
}
