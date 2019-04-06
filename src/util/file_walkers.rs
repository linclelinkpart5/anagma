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

/// Generic walker that supports either visiting parent or child files of an origin path.
pub enum FileWalker<'p> {
    Parent(ParentFileWalker<'p>),
    Child(ChildFileWalker<'p>),
}

impl<'p> Iterator for FileWalker<'p> {
    type Item = Result<Cow<'p, Path>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            // Parent walkers cannot error, so this needs wrapping in a `Result`.
            &mut Self::Parent(ref mut fw) => fw.next().map(Result::Ok),
            &mut Self::Child(ref mut fw) => fw.next(),
        }
    }
}

impl<'p> FileWalker<'p> {
    pub fn delve(&mut self, selection: &Selection, sort_order: SortOrder) -> Result<(), Error> {
        match self {
            // Parent walkers do not have to delve, just no-op.
            &mut Self::Parent(..) => Ok(()),
            &mut Self::Child(ref mut fw) => fw.delve(selection, sort_order),
        }
    }

    pub fn new_parent_walker(origin_item_path: &'p Path) -> Self {
        ParentFileWalker::new(origin_item_path).into()
    }

    pub fn new_child_walker(origin_item_path: &'p Path) -> Self {
        ChildFileWalker::new(origin_item_path).into()
    }
}

impl<'p> From<ParentFileWalker<'p>> for FileWalker<'p> {
    fn from(fw: ParentFileWalker<'p>) -> Self {
        Self::Parent(fw)
    }
}

impl<'p> From<ChildFileWalker<'p>> for FileWalker<'p> {
    fn from(fw: ChildFileWalker<'p>) -> Self {
        Self::Child(fw)
    }
}

pub struct ParentFileWalker<'p>(Option<&'p Path>);

impl<'p> ParentFileWalker<'p> {
    pub fn new(origin_item_path: &'p Path) -> Self {
        Self(Some(origin_item_path))
    }
}

impl<'p> Iterator for ParentFileWalker<'p> {
    type Item = Cow<'p, Path>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0 {
            Some(p) => {
                let ret = Some(p);

                self.0 = p.parent();

                ret.map(Cow::Borrowed)
            },
            None => None,
        }
    }
}

pub struct ChildFileWalker<'p> {
    frontier: VecDeque<Result<Cow<'p, Path>, Error>>,
    last_processed_path: Option<Cow<'p, Path>>,
}

impl<'p> ChildFileWalker<'p> {
    pub fn new(origin_item_path: &'p Path) -> Self {
        let mut frontier = VecDeque::new();

        // Initialize the frontier with the origin item.
        frontier.push_back(Ok(Cow::Borrowed(origin_item_path)));

        Self {
            frontier,
            last_processed_path: None,
        }
    }

    pub fn delve(&mut self, selection: &Selection, sort_order: SortOrder) -> Result<(), Error> {
        // Manually delves into a directory, and adds its subitems to the frontier.
        if let Some(lpp) = self.last_processed_path.take() {
            // If the last processed path is a directory, add its children to the frontier.
            if lpp.is_dir() {
                match selection.select_in_dir_sorted(&lpp, sort_order) {
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

impl<'p> Iterator for ChildFileWalker<'p> {
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
    use super::ParentFileWalker;
    use super::ChildFileWalker;

    use config::selection::Selection;
    use config::sort_order::SortOrder;

    use test_util::TestUtil;

    #[test]
    fn test_parent_file_walker() {
        let root_dir = TestUtil::create_plain_fanout_test_dir("test_parent_file_walker", 3, 3);

        let start_path = root_dir.path().join("0").join("0_1").join("0_1_0");
        let mut walker = ParentFileWalker::new(&start_path);

        assert_eq!(walker.next().unwrap(), root_dir.path().join("0").join("0_1").join("0_1_0"));
        assert_eq!(walker.next().unwrap(), root_dir.path().join("0").join("0_1"));
        assert_eq!(walker.next().unwrap(), root_dir.path().join("0"));
        assert_eq!(walker.next().unwrap(), root_dir.path());
    }

    #[test]
    fn test_child_file_walker() {
        let root_dir = TestUtil::create_plain_fanout_test_dir("test_child_file_walker", 3, 3);

        let start_path = root_dir.path();

        // Skip the first file of each leaf directory.
        // NOTE: Recall that directories are always selected.
        let selection = Selection::from_patterns(vec!["*_*"], vec!["*_0"]).unwrap();
        let sort_order = SortOrder::Name;
        let mut walker = ChildFileWalker::new(&start_path);

        // We should get just the root value, since no delving has happened.
        assert_eq!(walker.next().unwrap().unwrap(), root_dir.path());
        assert!(walker.next().is_none());

        // std::thread::sleep_ms(100000);

        walker.delve(&selection, sort_order).unwrap();
        assert_eq!(walker.next().unwrap().unwrap(), root_dir.path().join("0"));
        assert_eq!(walker.next().unwrap().unwrap(), root_dir.path().join("1"));
        assert_eq!(walker.next().unwrap().unwrap(), root_dir.path().join("2"));
        assert!(walker.next().is_none());

        // This delve call opens up the most recently accessed directory.
        walker.delve(&selection, sort_order).unwrap();
        assert_eq!(walker.next().unwrap().unwrap(), root_dir.path().join("2").join("2_0"));
        assert_eq!(walker.next().unwrap().unwrap(), root_dir.path().join("2").join("2_1"));

        walker.delve(&selection, sort_order).unwrap();
        assert_eq!(walker.next().unwrap().unwrap(), root_dir.path().join("2").join("2_1").join("2_1_0"));

        // Once files are found, observe the results of the selection.
        walker.delve(&selection, sort_order).unwrap();
        assert_eq!(walker.next().unwrap().unwrap(), root_dir.path().join("2").join("2_1").join("2_1_0").join("2_1_0_1"));
        assert_eq!(walker.next().unwrap().unwrap(), root_dir.path().join("2").join("2_1").join("2_1_0").join("2_1_0_2"));

        // Delving on a file does nothing.
        walker.delve(&selection, sort_order).unwrap();

        // Right back to where we were before delving into depth 3.
        assert_eq!(walker.next().unwrap().unwrap(), root_dir.path().join("2").join("2_1").join("2_1_1"));
        assert_eq!(walker.next().unwrap().unwrap(), root_dir.path().join("2").join("2_1").join("2_1_2"));
        assert_eq!(walker.next().unwrap().unwrap(), root_dir.path().join("2").join("2_2"));
        assert!(walker.next().is_none());
    }
}
