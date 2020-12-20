use std::borrow::Cow;
use std::fs::ReadDir;
use std::io::Result as IoResult;
use std::iter::Once;
use std::path::Path;

pub(crate) enum ItemPathsInner<'a> {
    ReadDir(ReadDir),
    Single(Once<&'a Path>),
}

impl<'a> Iterator for ItemPathsInner<'a> {
    type Item = IoResult<Cow<'a, Path>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::ReadDir(rd) => Some(rd.next()?.map(|e| Cow::Owned(e.path()))),
            Self::Single(o) => o.next().map(|p| Ok(Cow::Borrowed(p))),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::ReadDir(rd) => rd.size_hint(),
            Self::Single(o) => o.size_hint(),
        }
    }
}

pub struct ItemPaths<'a>(pub(crate) ItemPathsInner<'a>);

impl<'a> Iterator for ItemPaths<'a> {
    type Item = IoResult<Cow<'a, Path>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}
