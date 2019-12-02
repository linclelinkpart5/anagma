pub mod matcher;

use std::path::Path;
use std::path::PathBuf;

use crate::config::selection::matcher::Matcher;
use crate::config::sort_order::SortOrder;
use crate::config::selection::matcher::Error as MatcherError;

#[derive(Debug)]
pub enum Error {
    InvalidDirPath(PathBuf),
    CannotBuildMatcher(MatcherError),
    CannotReadDir(std::io::Error),
    CannotReadDirEntry(std::io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::InvalidDirPath(ref p) => write!(f, "not a valid directory: {}", p.display()),
            Error::CannotBuildMatcher(ref err) => write!(f, "cannot build matcher: {}", err),
            Error::CannotReadDir(ref err) => write!(f, "cannot read directory: {}", err),
            Error::CannotReadDirEntry(ref err) => write!(f, "cannot read directory entry: {}", err),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Error::InvalidDirPath(..) => None,
            Error::CannotBuildMatcher(ref err) => Some(err),
            Error::CannotReadDir(ref err) => Some(err),
            Error::CannotReadDirEntry(ref err) => Some(err),
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct Selection {
    include_files: Matcher,
    exclude_files: Matcher,
    include_dirs: Matcher,
    exclude_dirs: Matcher,
}

impl Default for Selection {
    fn default() -> Self {
        Selection {
            include_files: Matcher::any(),
            exclude_files: Matcher::from_patterns(&["item*", "self*"]).unwrap(),
            include_dirs: Matcher::any(),
            exclude_dirs: Matcher::empty(),
        }
    }
}

impl Selection {
    pub fn new(
        include_files: Matcher,
        exclude_files: Matcher,
        include_dirs: Matcher,
        exclude_dirs: Matcher,
    ) -> Self {
        Selection {
            include_files,
            exclude_files,
            include_dirs,
            exclude_dirs,
        }
    }

    pub fn from_patterns<FI, FIS, FE, FES, DI, DIS, DE, DES>(
        include_files_pattern_strs: FI,
        exclude_files_pattern_strs: FE,
        include_dirs_pattern_strs: DI,
        exclude_dirs_pattern_strs: DE,
    ) -> Result<Self, Error>
    where
        FI: IntoIterator<Item = FIS>,
        FIS: AsRef<str>,
        FE: IntoIterator<Item = FES>,
        FES: AsRef<str>,
        DI: IntoIterator<Item = DIS>,
        DIS: AsRef<str>,
        DE: IntoIterator<Item = DES>,
        DES: AsRef<str>,
    {
        let include_files = Matcher::from_patterns(include_files_pattern_strs).map_err(Error::CannotBuildMatcher)?;
        let exclude_files = Matcher::from_patterns(exclude_files_pattern_strs).map_err(Error::CannotBuildMatcher)?;
        let include_dirs = Matcher::from_patterns(include_dirs_pattern_strs).map_err(Error::CannotBuildMatcher)?;
        let exclude_dirs = Matcher::from_patterns(exclude_dirs_pattern_strs).map_err(Error::CannotBuildMatcher)?;

        Ok(Self::new(include_files, exclude_files, include_dirs, exclude_dirs))
    }

    /// Returns true if the path is a pattern match.
    /// In order to be a pattern match, the path must match the include filter, and must NOT match the exclude filter.
    /// This uses only the lexical content of the path, and does not access the filesystem.
    /// However, a flag is needed in order to determine whether this is matched as a file or as a directory.
    pub fn is_pattern_match<P: AsRef<Path>>(&self, path: P, is_file: bool) -> bool {
        let (inc, exc) =
            if is_file { (&self.include_files, &self.exclude_files) }
            else { (&self.include_dirs, &self.exclude_dirs) }
        ;

        inc.is_match(&path) && !exc.is_match(&path)
    }

    /// Returns true if a path is selected.
    /// Directories are always marked as selected.
    /// Files are selected if they are also a pattern match.
    pub fn is_selected<P: AsRef<Path>>(&self, path: P) -> bool {
        self.is_pattern_match(path.as_ref(), path.as_ref().is_file())
    }

    /// Returns items from the input iterable that are selected.
    pub fn select<'a, II, P>(&'a self, item_paths: II) -> impl Iterator<Item = P> + 'a
    where
        II: IntoIterator<Item = P>,
        II::IntoIter: 'a,
        P: AsRef<Path>,
    {
        let filtered = item_paths
            .into_iter()
            .filter(move |ip| self.is_selected(ip));

        filtered
    }

    pub fn select_in_dir<'a, P>(&'a self, dir_path: P) -> Result<impl Iterator<Item = PathBuf> + 'a, Error>
    where
        P: AsRef<Path>,
    {
        let dir_path = dir_path.as_ref();

        if !dir_path.is_dir() {
            return Err(Error::InvalidDirPath(dir_path.to_path_buf()));
        }

        let item_entries = dir_path
            .read_dir().map_err(Error::CannotReadDir)?
            .collect::<Result<Vec<_>, _>>().map_err(Error::CannotReadDirEntry)?;

        let sel_item_paths = self.select::<'a>(item_entries.into_iter().map(|entry| entry.path()));

        Ok(sel_item_paths)
    }

    pub fn select_in_dir_sorted<P>(&self, dir_path: P, sort_order: SortOrder) -> Result<Vec<PathBuf>, Error>
    where
        P: AsRef<Path>,
    {
        let mut sel_item_paths: Vec<_> = self.select_in_dir(dir_path)?.collect();
        sel_item_paths.sort_by(|a, b| sort_order.path_sort_cmp(a, b));

        Ok(sel_item_paths)
    }
}

#[cfg(test)]
mod tests {
    use super::Selection;

    use std::fs::File;

    use tempfile::Builder;
    use tempfile::TempDir;

    use crate::config::sort_order::SortOrder;

    fn create_test_dir(name: &str) -> TempDir {
        let temp = Builder::new().suffix(name).tempdir().expect("unable to create temp directory");

        {
            let path = temp.path();

            let file_names = vec![
                "music.flac",
                "music.wav",
                "music.aac",
                "music.mp3",
                "music.ogg",
                "item",
                "self",
                "item.yml",
                "self.yml",
                "item.flac",
                "self.flac",
            ];

            for file_name in file_names {
                File::create(path.join(file_name)).expect("unable to create temp file");
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
        }

        temp
    }

    #[test]
    fn test_default() {
        let selection = Selection::default();

        assert!(selection.is_pattern_match("all", true));
        assert!(selection.is_pattern_match("files", true));
        assert!(selection.is_pattern_match("should", true));
        assert!(selection.is_pattern_match("pass", true));
        assert!(selection.is_pattern_match("except", true));
        assert!(selection.is_pattern_match("for", true));
        assert!(selection.is_pattern_match("these", true));

        assert!(!selection.is_pattern_match("item", true));
        assert!(!selection.is_pattern_match("self", true));

        assert!(selection.is_pattern_match("all", false));
        assert!(selection.is_pattern_match("dirs", false));
        assert!(selection.is_pattern_match("should", false));
        assert!(selection.is_pattern_match("pass", false));
        assert!(selection.is_pattern_match("even", false));
        assert!(selection.is_pattern_match("including", false));
        assert!(selection.is_pattern_match("item", false));
        assert!(selection.is_pattern_match("self", false));
    }

    #[test]
    fn test_deserialization() {
        // A single pattern for each of include and exclude.
        let text = "include_files: '*.flac'\nexclude_files: '*.mp3'";
        let selection: Selection = serde_yaml::from_str(&text).unwrap();

        assert_eq!(selection.is_pattern_match("path/to/music.flac", true), true);
        assert_eq!(selection.is_pattern_match("path/to/music.mp3", true), false);
        assert_eq!(selection.is_pattern_match("path/to/music.wav", true), false);

        // Multiple patterns for each of include and exclude.
        let text = "include_files:\n  - '*.flac'\n  - '*.wav'\nexclude_files:\n  - '*.mp3'\n  - '*.ogg'";
        let selection: Selection = serde_yaml::from_str(&text).unwrap();

        assert_eq!(selection.is_pattern_match("path/to/music.flac", true), true);
        assert_eq!(selection.is_pattern_match("path/to/music.wav", true), true);
        assert_eq!(selection.is_pattern_match("path/to/music.mp3", true), false);
        assert_eq!(selection.is_pattern_match("path/to/music.ogg", true), false);
        assert_eq!(selection.is_pattern_match("path/to/music.aac", true), false);

        // Using a default value for missing include patterns.
        let text = "exclude_files:\n  - '*.mp3'\n  - '*.ogg'";
        let selection: Selection = serde_yaml::from_str(&text).unwrap();

        assert_eq!(selection.is_pattern_match("path/to/music.flac", true), true);
        assert_eq!(selection.is_pattern_match("path/to/music.wav", true), true);
        assert_eq!(selection.is_pattern_match("path/to/music.aac", true), true);
        assert_eq!(selection.is_pattern_match("path/to/music.mpc", true), true);
        assert_eq!(selection.is_pattern_match("path/to/music.mp3", true), false);
        assert_eq!(selection.is_pattern_match("path/to/music.ogg", true), false);

        // Using a default value for missing exclude patterns.
        let text = "include_files:\n  - '*.flac'\n  - '*.wav'";
        let selection: Selection = serde_yaml::from_str(&text).unwrap();

        assert_eq!(selection.is_pattern_match("path/to/music.flac", true), true);
        assert_eq!(selection.is_pattern_match("path/to/music.wav", true), true);
        assert_eq!(selection.is_pattern_match("path/to/item.flac", true), false);
        assert_eq!(selection.is_pattern_match("path/to/self.flac", true), false);
        assert_eq!(selection.is_pattern_match("path/to/music.aac", true), false);
        assert_eq!(selection.is_pattern_match("path/to/music.mpc", true), false);
        assert_eq!(selection.is_pattern_match("path/to/music.mp3", true), false);
        assert_eq!(selection.is_pattern_match("path/to/music.ogg", true), false);
    }

    #[test]
    fn test_from_patterns() {
        // Positive test cases.
        assert!(Selection::from_patterns(&["*"], &["*"], &["*"], &["*"]).is_ok());
        assert!(Selection::from_patterns(&["*.a", "*.b"], &["*.a", "*.b"], &["*.a", "*.b"], &["*.a", "*.b"]).is_ok());
        assert!(Selection::from_patterns(&["?.a", "?.b"], &["?.a", "?.b"], &["?.a", "?.b"], &["?.a", "?.b"]).is_ok());
        assert!(Selection::from_patterns(&["*.a"], &["*.a"], &["*.a"], &["*.a"]).is_ok());
        assert!(Selection::from_patterns(&["**"], &["**"], &["**"], &["**"]).is_ok());
        assert!(Selection::from_patterns(&["a/**/b"], &["a/**/b"], &["a/**/b"], &["a/**/b"]).is_ok());
        assert!(Selection::from_patterns(&[""; 0], &[""; 0], &[""; 0], &[""; 0]).is_ok());
        assert!(Selection::from_patterns(&[""], &[""], &[""], &[""]).is_ok());
        assert!(Selection::from_patterns(&["[a-z]*.a"], &["[a-z]*.a"], &["[a-z]*.a"], &["[a-z]*.a"]).is_ok());
        assert!(Selection::from_patterns(&["**", "[a-z]*.a"], &["**", "[a-z]*.a"], &["**", "[a-z]*.a"], &["**", "[a-z]*.a"]).is_ok());
        assert!(Selection::from_patterns(&["[!abc]"], &["[!abc]"], &["[!abc]"], &["[!abc]"]).is_ok());
        assert!(Selection::from_patterns(&["[*]"], &["[*]"], &["[*]"], &["[*]"]).is_ok());
        assert!(Selection::from_patterns(&["[?]"], &["[?]"], &["[?]"], &["[?]"]).is_ok());
        assert!(Selection::from_patterns(&["{*.a,*.b,*.c}"], &["{*.a,*.b,*.c}"], &["{*.a,*.b,*.c}"], &["{*.a,*.b,*.c}"]).is_ok());

        // Negative test cases.
        // Invalid double star.
        // assert!(Selection::from_patterns(&["a**b"], &["a**b"], &["a**b"], &["a**b"]).is_err());
        // Unclosed character class.
        assert!(Selection::from_patterns(&["[abc"], &["[abc"], &["[abc"], &["[abc"]).is_err());
        // Malformed character range.
        assert!(Selection::from_patterns(&["[z-a]"], &["[z-a]"], &["[z-a]"], &["[z-a]"]).is_err());
        // Unclosed alternates.
        assert!(Selection::from_patterns(&["{*.a,*.b,*.c"], &["{*.a,*.b,*.c"], &["{*.a,*.b,*.c"], &["{*.a,*.b,*.c"]).is_err());
        // Unopened alternates.
        // assert!(Selection::from_patterns(&["*.a,*.b,*.c}"], &["*.a,*.b,*.c}"], &["*.a,*.b,*.c}"], &["*.a,*.b,*.c}"]).is_err());
        // Nested alternates.
        assert!(Selection::from_patterns(&["{*.a,{*.b,*.c}}"], &["{*.a,{*.b,*.c}}"], &["{*.a,{*.b,*.c}}"], &["{*.a,{*.b,*.c}}"]).is_err());
        // Dangling escape.
        assert!(Selection::from_patterns(&["*.a\\"], &["*.a\\"], &["*.a\\"], &["*.a\\"]).is_err());
    }

    #[test]
    fn test_is_pattern_match() {
        let selection = Selection::from_patterns(&["*.flac"], &["*.mp3"], &["*"], &[] as &[&str]).unwrap();

        assert_eq!(selection.is_pattern_match("path/to/music.flac", true), true);
        assert_eq!(selection.is_pattern_match("path/to/music.mp3", true), false);
        assert_eq!(selection.is_pattern_match("path/to/music.wav", true), false);

        let selection = Selection::from_patterns(&["*.flac", "*.wav"], &["*.mp3", "*.ogg"], &["*"], &[] as &[&str]).unwrap();

        assert_eq!(selection.is_pattern_match("path/to/music.flac", true), true);
        assert_eq!(selection.is_pattern_match("path/to/music.wav", true), true);
        assert_eq!(selection.is_pattern_match("path/to/music.mp3", true), false);
        assert_eq!(selection.is_pattern_match("path/to/music.ogg", true), false);
        assert_eq!(selection.is_pattern_match("path/to/music.aac", true), false);

        let selection = Selection::from_patterns(&["*"], &["*.mp3", "*.ogg"], &["*"], &[] as &[&str]).unwrap();

        assert_eq!(selection.is_pattern_match("path/to/music.flac", true), true);
        assert_eq!(selection.is_pattern_match("path/to/music.wav", true), true);
        assert_eq!(selection.is_pattern_match("path/to/music.aac", true), true);
        assert_eq!(selection.is_pattern_match("path/to/music.mpc", true), true);
        assert_eq!(selection.is_pattern_match("path/to/music.mp3", true), false);
        assert_eq!(selection.is_pattern_match("path/to/music.ogg", true), false);

        let selection = Selection::from_patterns(&["*.flac", "*.wav"], &["item*", "self*"], &["*"], &[] as &[&str]).unwrap();

        assert_eq!(selection.is_pattern_match("path/to/music.flac", true), true);
        assert_eq!(selection.is_pattern_match("path/to/music.wav", true), true);
        assert_eq!(selection.is_pattern_match("path/to/item.flac", true), false);
        assert_eq!(selection.is_pattern_match("path/to/self.flac", true), false);
        assert_eq!(selection.is_pattern_match("path/to/music.aac", true), false);
        assert_eq!(selection.is_pattern_match("path/to/music.mpc", true), false);
        assert_eq!(selection.is_pattern_match("path/to/music.mp3", true), false);
        assert_eq!(selection.is_pattern_match("path/to/music.ogg", true), false);
    }

    #[test]
    fn test_select_in_dir() {
        let temp_dir = create_test_dir("test_select_in_dir");
        let path = temp_dir.path();

        let inputs_and_expected = vec![
            (
                (vec!["music*"], vec!["*.mp3", "*.ogg", "*.aac"], vec!["*"], Vec::<&str>::new()),
                hashset![
                    path.join("music.flac"),
                    path.join("music.wav"),
                ],
            ),
            (
                (vec!["*.flac"], vec![], vec!["*"], Vec::<&str>::new()),
                hashset![
                    path.join("music.flac"),
                    path.join("item.flac"),
                    path.join("self.flac"),
                ],
            ),
            (
                (vec!["music*"], vec![], vec!["*"], Vec::<&str>::new()),
                hashset![
                    path.join("music.flac"),
                    path.join("music.wav"),
                    path.join("music.aac"),
                    path.join("music.mp3"),
                    path.join("music.ogg"),
                ],
            ),
            (
                (vec!["item.*", "self.*"], vec!["*.flac"], vec!["*"], Vec::<&str>::new()),
                hashset![
                    path.join("item.yml"),
                    path.join("self.yml"),
                ],
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let (include_file_patterns, exclude_file_patterns, include_dir_patterns, exclude_dir_patterns) = input;

            let selection = Selection::from_patterns(
                include_file_patterns,
                exclude_file_patterns,
                include_dir_patterns,
                exclude_dir_patterns,
            ).unwrap();
            let produced = selection.select_in_dir(&path).unwrap().collect();
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_select_in_dir_sorted() {
        let temp_dir = create_test_dir("test_select_in_dir_sorted");
        let path = temp_dir.path();

        let inputs_and_expected = vec![
            (
                (vec!["music*"], vec!["*.mp3", "*.ogg", "*.aac"], vec!["*"], Vec::<&str>::new(), SortOrder::Name),
                vec![
                    path.join("music.flac"),
                    path.join("music.wav"),
                ],
            ),
            (
                (vec!["*.flac"], vec![], vec!["*"], Vec::<&str>::new(), SortOrder::Name),
                vec![
                    path.join("item.flac"),
                    path.join("music.flac"),
                    path.join("self.flac"),
                ],
            ),
            (
                (vec!["music*"], vec![], vec!["*"], Vec::<&str>::new(), SortOrder::Name),
                vec![
                    path.join("music.aac"),
                    path.join("music.flac"),
                    path.join("music.mp3"),
                    path.join("music.ogg"),
                    path.join("music.wav"),
                ],
            ),
            (
                (vec!["item.*", "self.*"], vec!["*.flac"], vec!["*"], Vec::<&str>::new(), SortOrder::Name),
                vec![
                    path.join("item.yml"),
                    path.join("self.yml"),
                ],
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let (include_file_patterns, exclude_file_patterns, include_dir_patterns, exclude_dir_patterns, sort_order) = input;

            let selection = Selection::from_patterns(
                include_file_patterns,
                exclude_file_patterns,
                include_dir_patterns,
                exclude_dir_patterns,
            ).unwrap();
            let produced = selection.select_in_dir_sorted(&path, sort_order).expect("unable to select in dir");
            assert_eq!(expected, produced);
        }
    }
}
