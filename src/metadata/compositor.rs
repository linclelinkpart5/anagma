
use crate::metadata::schema::SchemaFormat;

enum Source<'a> {
    /// The metadata file location is a sibling of the target item file path.
    External(&'a str),

    /// The metadata file location is inside the target item file path.
    /// Implies that the the target item file path is a directory.
    Internal(&'a str),
}

pub struct Compositor<'a>(Vec<Source<'a>>);

impl<'a> Compositor<'a> {
    pub fn new<S: AsRef<str>>() -> Self {
        Self(Vec::new())
    }
}
