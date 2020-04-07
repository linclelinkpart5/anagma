use std::borrow::Cow;
use std::path::Path;
use std::path::PathBuf;
use std::collections::VecDeque;
use std::path::Ancestors;
use std::io::Error as IoError;

use crate::config::selection::Selection;
use crate::config::sorter::Sorter;

// #[derive(Debug)]
// pub enum Error {
//     CannotBulkSelect(IoError, Vec<IoError>),
// }

// impl std::fmt::Display for Error {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         match *self {
//             Self::CannotBulkSelect(_, ref other_errs)
//                 => write!(f, "errors when bulk selecting: {}", 1 + other_errs.len()),
//         }
//     }
// }

// impl std::error::Error for Error {
//     fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
//         match *self {
//             Self::CannotBulkSelect(ref err, _) => Some(err),
//         }
//     }
// }

/// Generic file walker that supports visiting either parent or child files of
/// an origin path.
#[derive(Debug)]
pub enum FileWalker<'p> {
    Parent(ParentFileWalker<'p>),
    Child(ChildFileWalker<'p>),
}

impl<'p> Iterator for FileWalker<'p> {
    type Item = Result<Cow<'p, Path>, IoError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            // Parent walkers cannot error, so this needs wrapping in a `Result`.
            Self::Parent(ref mut fw) => fw.next().map(Result::Ok),
            Self::Child(ref mut fw) => fw.next(),
        }
    }
}

impl<'p> FileWalker<'p> {
    pub fn delve(&mut self, selection: &Selection, sorter: Sorter) -> Result<(), IoError> {
        match self {
            // Parent walkers do not have to delve, just no-op.
            Self::Parent(..) => Ok(()),
            Self::Child(ref mut fw) => fw.delve(selection, sorter),
        }
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

/// A file walker that starts at an origin path, and walks up the directory tree.
#[derive(Debug)]
pub struct ParentFileWalker<'p>(Ancestors<'p>);

impl<'p> ParentFileWalker<'p> {
    /// Constructs a new `ParentFileWalker` starting at a specified item path.
    // LEARN: Since `PathBuf` impls `AsRef<Path>`, a caller could pass ownership
    // of a `PathBuf` here, so `&'p P` instead of just `P` is required.
    // This forces the input to be a borrow, so storing the result of `.as_ref()`
    // (which borrows its input) is valid.
    pub fn new<P: AsRef<Path>>(origin_item_path: &'p P) -> Self {
        Self(origin_item_path.as_ref().ancestors())
    }
}

impl<'p> Iterator for ParentFileWalker<'p> {
    type Item = Cow<'p, Path>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(Cow::Borrowed)
    }
}

/// A file walker that starts at an origin path, with the ability to delve
/// recursively into its directory structure to visit its children, grandchildren, etc.
#[derive(Debug)]
pub struct ChildFileWalker<'p> {
    frontier: VecDeque<Result<Cow<'p, Path>, IoError>>,
    last_processed_path: Option<Cow<'p, Path>>,
}

impl<'p> ChildFileWalker<'p> {
    /// Constructs a new `ChildFileWalker` starting at a specified item path.
    pub fn new<P: AsRef<Path>>(origin_item_path: &'p P) -> Self {
        let mut frontier = VecDeque::new();

        // Initialize the frontier with the origin item.
        frontier.push_back(Ok(Cow::Borrowed(origin_item_path.as_ref())));

        let last_processed_path = None;

        Self { frontier, last_processed_path, }
    }

    /// Manually delves into a directory, and adds its subitems to the frontier.
    /// Note that this is a no-op if the most recent processed path is not a
    /// directory, and not an error.
    pub fn delve(&mut self, selection: &Selection, sorter: Sorter) -> Result<(), IoError> {
        // If there is a last processed path, delve into it.
        // If not, just no-op.
        if let Some(lpp) = self.last_processed_path.take() {
            // Get file info for the last processed path.
            let file_info = std::fs::metadata(&lpp)?;

            // Only work on directories.
            if file_info.is_dir() {
                let mut sub_item_paths = selection.select_in_dir_sorted(&lpp, sorter)?;

                // NOTE: Reversing and pushing onto the front of the queue is needed.
                for p in sub_item_paths.drain(..).rev() {
                    self.frontier.push_front(p.map(Cow::Owned));
                }
            }
        }

        Ok(())
    }
}

impl<'p> Iterator for ChildFileWalker<'p> {
    type Item = Result<Cow<'p, Path>, IoError>;

    fn next(&mut self) -> Option<Self::Item> {
        let frontier_item_result = self.frontier.pop_front()?;

        // Save the most recently processed item path, if any.
        if let Ok(frontier_item_path) = frontier_item_result.as_ref() {
            self.last_processed_path = Some(frontier_item_path.clone());
        }

        Some(frontier_item_result)
    }
}

#[cfg(test)]
mod tests {
    use super::ParentFileWalker;
    use super::ChildFileWalker;

    use crate::config::selection::Selection;
    use crate::config::sorter::Sorter;

    use crate::test_util::TestUtil;

    #[test]
    fn parent_file_walker() {
        let root_dir = TestUtil::create_plain_fanout_test_dir("test_parent_file_walker", 3, 3);

        let start_path = root_dir.path().join("0").join("0_1").join("0_1_0");
        let mut walker = ParentFileWalker::new(&start_path);

        assert_eq!(walker.next().unwrap(), root_dir.path().join("0").join("0_1").join("0_1_0"));
        assert_eq!(walker.next().unwrap(), root_dir.path().join("0").join("0_1"));
        assert_eq!(walker.next().unwrap(), root_dir.path().join("0"));
        assert_eq!(walker.next().unwrap(), root_dir.path());
    }

    #[test]
    fn child_file_walker() {
        let root_dir = TestUtil::create_plain_fanout_test_dir("test_child_file_walker", 3, 3);

        let start_path = root_dir.path();

        // Skip the first file of each leaf directory.
        let selection = Selection::from_patterns(&["*_*"], &["*_0"], &["*"], &[] as &[&str]).unwrap();
        let sorter = Sorter::default();
        let mut walker = ChildFileWalker::new(&start_path);

        // We should get just the root value, since no delving has happened.
        assert_eq!(walker.next().unwrap().unwrap(), root_dir.path());
        assert!(walker.next().is_none());

        walker.delve(&selection, sorter);
        assert_eq!(walker.next().unwrap().unwrap(), root_dir.path().join("0"));
        assert_eq!(walker.next().unwrap().unwrap(), root_dir.path().join("1"));
        assert_eq!(walker.next().unwrap().unwrap(), root_dir.path().join("2"));
        assert!(walker.next().is_none());

        // This delve call opens up the most recently accessed directory.
        walker.delve(&selection, sorter);
        assert_eq!(walker.next().unwrap().unwrap(), root_dir.path().join("2").join("2_0"));
        assert_eq!(walker.next().unwrap().unwrap(), root_dir.path().join("2").join("2_1"));

        walker.delve(&selection, sorter);
        assert_eq!(walker.next().unwrap().unwrap(), root_dir.path().join("2").join("2_1").join("2_1_0"));

        // Once files are found, observe the results of the selection.
        walker.delve(&selection, sorter);
        assert_eq!(walker.next().unwrap().unwrap(), root_dir.path().join("2").join("2_1").join("2_1_0").join("2_1_0_1"));
        assert_eq!(walker.next().unwrap().unwrap(), root_dir.path().join("2").join("2_1").join("2_1_0").join("2_1_0_2"));

        // Delving on a file does nothing, and does not error.
        walker.delve(&selection, sorter);

        // Right back to where we were before delving into depth 3.
        assert_eq!(walker.next().unwrap().unwrap(), root_dir.path().join("2").join("2_1").join("2_1_1"));
        assert_eq!(walker.next().unwrap().unwrap(), root_dir.path().join("2").join("2_1").join("2_1_2"));
        assert_eq!(walker.next().unwrap().unwrap(), root_dir.path().join("2").join("2_2"));
        assert!(walker.next().is_none());
    }
}
