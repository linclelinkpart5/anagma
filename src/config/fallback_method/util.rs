use std::path::Path;
use std::path::PathBuf;
use std::collections::VecDeque;

use config::meta_format::MetaFormat;
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
            Error::Selection(ref err) => write!(f, "selection error: {}", err),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Error::Selection(ref err) => Some(err),
        }
    }
}

struct AncestorItemVisitor<'p>(Option<&'p Path>);

impl<'p> AncestorItemVisitor<'p> {
    pub fn new(origin_item_path: &'p Path) -> Self {
        Self(origin_item_path.parent())
    }
}

impl<'p> Iterator for AncestorItemVisitor<'p> {
    type Item = &'p Path;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0 {
            Some(p) => {
                let ret = Some(p);

                self.0 = p.parent();

                ret
            },
            None => None,
        }
    }
}

struct ChildrenItemVisitor<'s> {
    frontier: VecDeque<Result<PathBuf, Error>>,
    last_processed_path: Option<PathBuf>,

    meta_format: MetaFormat,
    selection: &'s Selection,
    sort_order: SortOrder,
}

impl<'s> ChildrenItemVisitor<'s> {
    pub fn new<P: AsRef<Path>>(
        origin_item_path: P,
        meta_format: MetaFormat,
        selection: &'s Selection,
        sort_order: SortOrder,
    ) -> Self
    {
        let mut frontier = VecDeque::new();

        // Initialize the frontier with the subitems of the origin.
        match selection.select_in_dir_sorted(origin_item_path, sort_order) {
            // TODO: Return Result<PathBuf, *> instead of just PathBuf.
            Err(err) => {
                frontier.push_back(Err(Error::Selection(err)));
            },
            Ok(mut sub_item_paths) => {
                for p in sub_item_paths.drain(..) {
                    frontier.push_back(Ok(p));
                }
            },
        };

        Self {
            frontier,
            last_processed_path: None,
            meta_format,
            selection,
            sort_order,
        }
    }

    pub fn delve(&mut self) {
        // Manually delves into a directory, and adds its subitems to the frontier.
        if let Some(lpp) = self.last_processed_path.take() {
            // If the last processed path is a directory, add its children to the frontier.
            if lpp.is_dir() {
                match self.selection.select_in_dir_sorted(&lpp, self.sort_order) {
                    Err(err) => {
                        self.frontier.push_back(Err(Error::Selection(err)));
                    },
                    Ok(mut sub_item_paths) => {
                        for p in sub_item_paths.drain(..).rev() {
                            self.frontier.push_front(Ok(p));
                        }
                    },
                }
            }
        }
    }
}

impl<'s> Iterator for ChildrenItemVisitor<'s> {
    type Item = Result<PathBuf, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(frontier_item_result) = self.frontier.pop_front() {
            // Save the most recently processed item path, if any.
            if let Ok(frontier_item_path) = frontier_item_result.as_ref() {
                self.last_processed_path = Some(frontier_item_path.clone());
            }

            Some(frontier_item_result)
        }
        else {
            None
        }
    }
}
