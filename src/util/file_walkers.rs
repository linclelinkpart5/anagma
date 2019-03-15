use std::borrow::Cow;
use std::path::Path;
use std::collections::VecDeque;

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

struct AncestorFileWalker<'p>(Option<&'p Path>);

impl<'p> AncestorFileWalker<'p> {
    pub fn new(origin_item_path: &'p Path) -> Self {
        Self(Some(origin_item_path))
    }
}

impl<'p> Iterator for AncestorFileWalker<'p> {
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

struct ChildrenFileWalker<'p, 's> {
    frontier: VecDeque<Result<Cow<'p, Path>, Error>>,
    last_processed_path: Option<Cow<'p, Path>>,

    selection: &'s Selection,
    sort_order: SortOrder,
}

impl<'p, 's> ChildrenFileWalker<'p, 's> {
    pub fn new(origin_item_path: &'p Path, selection: &'s Selection, sort_order: SortOrder) -> Self {
        let mut frontier = VecDeque::new();

        // Initialize the frontier with the origin item.
        frontier.push_back(Ok(Cow::Borrowed(origin_item_path)));

        Self {
            frontier,
            last_processed_path: None,
            selection,
            sort_order,
        }
    }

    pub fn delve(&mut self) -> Result<(), Error> {
        // Manually delves into a directory, and adds its subitems to the frontier.
        if let Some(lpp) = self.last_processed_path.take() {
            // If the last processed path is a directory, add its children to the frontier.
            if lpp.is_dir() {
                match self.selection.select_in_dir_sorted(&lpp, self.sort_order) {
                    Err(err) => {
                        return Err(Error::Selection(err));
                    },
                    Ok(mut sub_item_paths) => {
                        // NOTE: Reversing and pushing onto the front of the queue is needed.
                        for p in sub_item_paths.drain(..).rev() {
                            self.frontier.push_front(Ok(Cow::Owned(p)));
                        }
                    },
                }
            }
        }

        Ok(())
    }
}

impl<'p, 's> Iterator for ChildrenFileWalker<'p, 's> {
    type Item = Result<Cow<'p, Path>, Error>;

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

#[cfg(test)]
mod tests {
    use super::AncestorFileWalker;
    use super::ChildrenFileWalker;

    use std::path::Path;
    use std::fs::DirBuilder;
    use std::fs::File;

    use tempfile::Builder;
    use tempfile::TempDir;

    use config::selection::Selection;
    use config::sort_order::SortOrder;

    const FANOUT: u8 = 3;
    const MAX_DEPTH: u8 = 4;

    fn create_dir_tree(name: &str) -> TempDir {
        let root_dir = Builder::new().suffix(name).tempdir().expect("unable to create temp directory");

        fn fill_dir(p: &Path, db: &DirBuilder, curr_depth: u8) {
            for i in 0..FANOUT {
                let name = format!("{}_{}", curr_depth, i);
                let new_path = p.join(&name);

                if curr_depth >= MAX_DEPTH {
                    // Create files.
                    File::create(&new_path).unwrap();
                }
                else {
                    // Create dirs and then recurse.
                    db.create(&new_path).unwrap();
                    fill_dir(&new_path, &db, curr_depth + 1);
                }
            }
        }

        let db = DirBuilder::new();

        fill_dir(root_dir.path(), &db, 0);

        root_dir
    }

    #[test]
    fn test_ancestor_file_walker() {
        let root_dir = create_dir_tree("test_ancestor_file_walker");

        let start_path = root_dir.path().join("0_0").join("1_0").join("2_0");
        let mut walker = AncestorFileWalker::new(&start_path);

        assert_eq!(walker.next().unwrap().unwrap(), root_dir.path().join("0_0").join("1_0").join("2_0"));
        assert_eq!(walker.next().unwrap().unwrap(), root_dir.path().join("0_0").join("1_0"));
        assert_eq!(walker.next().unwrap().unwrap(), root_dir.path().join("0_0"));
        assert_eq!(walker.next().unwrap().unwrap(), root_dir.path());
    }

    #[test]
    fn test_children_file_walker() {
        let root_dir = create_dir_tree("test_children_file_walker");

        let start_path = root_dir.path();

        // Skip the first file of each leaf directory.
        // NOTE: Recall that directories are always selected.
        let selection = Selection::from_patterns(vec!["*_*"], vec!["*_0"]).unwrap();
        let mut walker = ChildrenFileWalker::new(&start_path, &selection, SortOrder::Name);

        // We should get just the root value, since no delving has happened.
        assert_eq!(walker.next().unwrap().unwrap(), root_dir.path());
        assert!(walker.next().is_none());

        walker.delve().unwrap();
        assert_eq!(walker.next().unwrap().unwrap(), root_dir.path().join("0_0"));
        assert_eq!(walker.next().unwrap().unwrap(), root_dir.path().join("0_1"));
        assert_eq!(walker.next().unwrap().unwrap(), root_dir.path().join("0_2"));

        // This delve call opens up the most recently accessed directory.
        walker.delve().unwrap();
        assert_eq!(walker.next().unwrap().unwrap(), root_dir.path().join("0_2").join("1_0"));
        assert_eq!(walker.next().unwrap().unwrap(), root_dir.path().join("0_2").join("1_1"));
        assert_eq!(walker.next().unwrap().unwrap(), root_dir.path().join("0_2").join("1_2"));
    }
}
