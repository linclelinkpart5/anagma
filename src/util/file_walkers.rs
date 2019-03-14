use std::borrow::Cow;
use std::path::Path;

use walkdir::WalkDir;
use walkdir::FilterEntry;
use walkdir::IntoIter;

use config::selection::Selection;
use config::selection::Error as SelectionError;
use config::sort_order::SortOrder;

#[derive(Debug)]
pub enum Error {
    Selection(SelectionError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::Selection(ref err) => write!(f, "selection error: {}", err),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Self::Selection(ref err) => Some(err),
        }
    }
}

pub(crate) struct PItemIter<'p>(Option<&'p Path>);

impl<'p> PItemIter<'p> {
    pub fn new(origin_item_path: &'p Path) -> Self {
        Self(Some(origin_item_path))
    }
}

impl<'p> Iterator for PItemIter<'p> {
    type Item = Result<Cow<'p, Path>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0 {
            Some(p) => {
                let ret = Some(p);

                self.0 = p.parent();

                ret.map(Cow::Borrowed).map(Result::Ok)
            },
            None => None,
        }
    }
}

pub(crate) struct CItemIter<'s>(IntoIter, &'s Selection);

impl<'s> CItemIter<'s> {
    pub fn new(origin_item_path: &Path, selection: &'s Selection, sort_order: SortOrder) -> Self {
        Self(
            WalkDir::new(origin_item_path)
                .follow_links(true)
                .sort_by(move |a, b| sort_order.path_sort_cmp(a.path(), b.path()))
                .into_iter(),
            selection,
        )
    }
}

// impl Iterator for CItemIter {
//     type Item = Result<Cow<'p, Path>, Error>;

//     fn next(&mut self) -> Option<Self::Item> {
//         match self.0 {
//             Some(p) => {
//                 let ret = Some(p);

//                 self.0 = p.parent();

//                 ret.map(Cow::Borrowed).map(Result::Ok)
//             },
//             None => None,
//         }
//     }
// }
