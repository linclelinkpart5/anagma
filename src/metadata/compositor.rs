
use std::ffi::OsString;

use crate::metadata::schema::SchemaFormat;

pub(crate) enum Source {
    /// The metadata file location is a sibling of the target item file path.
    External(OsString),

    /// The metadata file location is inside the target item file path.
    /// Implies that the the target item file path is a directory.
    Internal(OsString),
}

pub struct Compositor(Vec<Source>, SchemaFormat);

impl<'a> Compositor {
    pub(crate) fn new(fmt: SchemaFormat) -> Self {
        Self(Vec::new(), fmt)
    }

    fn _add_src<I, F>(&mut self, file_stub: I, f: F) -> &mut Self
    where
        I: Into<OsString>,
        F: Fn(OsString) -> Source,
    {
        let mut src_fn = file_stub.into();
        src_fn.push(".");
        src_fn.push(self.1.file_extension());

        let src = f(src_fn);

        self.0.push(src);
        self
    }

    pub(crate) fn ex_source<I: Into<OsString>>(&mut self, file_stub: I) -> &mut Self {
        self._add_src(file_stub, Source::External)
    }

    pub(crate) fn in_source<I: Into<OsString>>(&mut self, file_stub: I) -> &mut Self {
        self._add_src(file_stub, Source::Internal)
    }
}
