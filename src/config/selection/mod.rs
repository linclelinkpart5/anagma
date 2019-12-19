pub mod matcher;

use std::path::Path;
use std::path::PathBuf;

use crate::strum::IntoEnumIterator;

use crate::config::selection::matcher::Matcher;
use crate::config::selection::matcher::Error as MatcherError;
use crate::config::sorter::Sorter;

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

enum FileOrDir {
    File,
    Dir,
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
        use crate::metadata::target::Target;

        // TODO: Replace with `StaticVec` once released for stable Rust.
        let excluded_patterns = Target::iter()
            .map(|ml| format!("{}{}", ml.default_file_name(), "*"))
            .collect::<Vec<_>>()
        ;

        let include_files = Matcher::any();
        let exclude_files = Matcher::build(&excluded_patterns).unwrap();
        let include_dirs = Matcher::any();
        let exclude_dirs = Matcher::empty();

        Self { include_files, exclude_files, include_dirs, exclude_dirs, }
    }
}

impl Selection {
    pub fn new(
        include_files: Matcher,
        exclude_files: Matcher,
        include_dirs: Matcher,
        exclude_dirs: Matcher,
    ) -> Self
    {
        Self { include_files, exclude_files, include_dirs, exclude_dirs, }
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
        let include_files = Matcher::build(include_files_pattern_strs).map_err(Error::CannotBuildMatcher)?;
        let exclude_files = Matcher::build(exclude_files_pattern_strs).map_err(Error::CannotBuildMatcher)?;
        let include_dirs = Matcher::build(include_dirs_pattern_strs).map_err(Error::CannotBuildMatcher)?;
        let exclude_dirs = Matcher::build(exclude_dirs_pattern_strs).map_err(Error::CannotBuildMatcher)?;

        Ok(Self::new(include_files, exclude_files, include_dirs, exclude_dirs))
    }

    fn is_pattern_match<P: AsRef<Path>>(&self, path: P, fod: FileOrDir) -> bool {
        let (inc, exc) = match fod {
            FileOrDir::File => (&self.include_files, &self.exclude_files),
            FileOrDir::Dir => (&self.include_dirs, &self.exclude_dirs),
        };

        inc.is_match(&path) && !exc.is_match(&path)
    }

    /// Returns true if the path pattern matches according to the file matcher.
    /// In order to be a pattern match, the path must match the include filter,
    /// and must NOT match the exclude filter.
    /// Note that this method assumes the path is a file, and uses only the
    /// lexical content of the path; it does not access the filesystem.
    pub fn is_file_pattern_match<P: AsRef<Path>>(&self, path: P) -> bool {
        self.is_pattern_match(path, FileOrDir::File)
    }

    /// Returns true if the path pattern matches according to the directory matcher.
    /// In order to be a pattern match, the path must match the include filter,
    /// and must NOT match the exclude filter.
    /// Note that this method assumes the path is a directory, and uses only the
    /// lexical content of the path; it does not access the filesystem.
    pub fn is_dir_pattern_match<P: AsRef<Path>>(&self, path: P) -> bool {
        self.is_pattern_match(path, FileOrDir::Dir)
    }

    /// Returns true if a path is selected.
    /// This accesses the filesystem to tell if the path is a file or directory.
    pub fn is_selected<P: AsRef<Path>>(&self, path: P) -> bool {
        match path.as_ref().is_dir() {
            false => self.is_file_pattern_match(path),
            true => self.is_dir_pattern_match(path),
        }
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

    pub fn select_in_dir_sorted<P>(&self, dir_path: P, sorter: Sorter) -> Result<Vec<PathBuf>, Error>
    where
        P: AsRef<Path>,
    {
        let mut sel_item_paths: Vec<_> = self.select_in_dir(dir_path)?.collect();
        sel_item_paths.sort_by(|a, b| sorter.path_sort_cmp(a, b));

        Ok(sel_item_paths)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs::File;

    use tempfile::Builder;
    use tempfile::TempDir;

    use crate::config::sorter::Sorter;

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

        assert!(selection.is_file_pattern_match("all"));
        assert!(selection.is_file_pattern_match("files"));
        assert!(selection.is_file_pattern_match("should"));
        assert!(selection.is_file_pattern_match("pass"));
        assert!(selection.is_file_pattern_match("except"));
        assert!(selection.is_file_pattern_match("for"));
        assert!(selection.is_file_pattern_match("these"));
        assert!(!selection.is_file_pattern_match("item",));
        assert!(!selection.is_file_pattern_match("self",));

        assert!(selection.is_dir_pattern_match("all"));
        assert!(selection.is_dir_pattern_match("dirs"));
        assert!(selection.is_dir_pattern_match("should"));
        assert!(selection.is_dir_pattern_match("pass"));
        assert!(selection.is_dir_pattern_match("even"));
        assert!(selection.is_dir_pattern_match("including"));
        assert!(selection.is_dir_pattern_match("item"));
        assert!(selection.is_dir_pattern_match("self"));
    }

    #[test]
    fn test_deserialization() {
        // A single pattern for each of include and exclude.
        let text = "{ include_files: '*.flac', exclude_files: '*.mp3' }";
        let selection: Selection = serde_yaml::from_str(&text).unwrap();

        assert_eq!(selection.is_file_pattern_match("path/to/music.flac"), true);
        assert_eq!(selection.is_file_pattern_match("path/to/music.mp3"), false);
        assert_eq!(selection.is_file_pattern_match("path/to/music.wav"), false);

        // Multiple patterns for each of include and exclude.
        let text = "{ include_files: ['*.flac', '*.wav'], exclude_files: ['*.mp3', '*.ogg'] }";
        let selection: Selection = serde_yaml::from_str(&text).unwrap();

        assert_eq!(selection.is_file_pattern_match("path/to/music.flac"), true);
        assert_eq!(selection.is_file_pattern_match("path/to/music.wav"), true);
        assert_eq!(selection.is_file_pattern_match("path/to/music.mp3"), false);
        assert_eq!(selection.is_file_pattern_match("path/to/music.ogg"), false);
        assert_eq!(selection.is_file_pattern_match("path/to/music.aac"), false);

        // Using a default value for missing include patterns.
        let text = "{ exclude_files: ['*.mp3', '*.ogg'] }";
        let selection: Selection = serde_yaml::from_str(&text).unwrap();

        assert_eq!(selection.is_file_pattern_match("path/to/music.flac"), true);
        assert_eq!(selection.is_file_pattern_match("path/to/music.wav"), true);
        assert_eq!(selection.is_file_pattern_match("path/to/music.aac"), true);
        assert_eq!(selection.is_file_pattern_match("path/to/music.mpc"), true);
        assert_eq!(selection.is_file_pattern_match("path/to/music.mp3"), false);
        assert_eq!(selection.is_file_pattern_match("path/to/music.ogg"), false);

        // Using a default value for missing exclude patterns.
        let text = "{ include_files: ['*.flac', '*.wav'] }";
        let selection: Selection = serde_yaml::from_str(&text).unwrap();

        assert_eq!(selection.is_file_pattern_match("path/to/music.flac"), true);
        assert_eq!(selection.is_file_pattern_match("path/to/music.wav"), true);
        assert_eq!(selection.is_file_pattern_match("path/to/item.flac"), false);
        assert_eq!(selection.is_file_pattern_match("path/to/self.flac"), false);
        assert_eq!(selection.is_file_pattern_match("path/to/music.aac"), false);
        assert_eq!(selection.is_file_pattern_match("path/to/music.mpc"), false);
        assert_eq!(selection.is_file_pattern_match("path/to/music.mp3"), false);
        assert_eq!(selection.is_file_pattern_match("path/to/music.ogg"), false);
    }

    #[test]
    fn test_is_pattern_match() {
        let selection = Selection::from_patterns(&["*.flac"], &["*.mp3"], &["*"], &[] as &[&str]).unwrap();

        assert_eq!(selection.is_file_pattern_match("path/to/music.flac"), true);
        assert_eq!(selection.is_file_pattern_match("path/to/music.mp3"), false);
        assert_eq!(selection.is_file_pattern_match("path/to/music.wav"), false);

        let selection = Selection::from_patterns(&["*.flac", "*.wav"], &["*.mp3", "*.ogg"], &["*"], &[] as &[&str]).unwrap();

        assert_eq!(selection.is_file_pattern_match("path/to/music.flac"), true);
        assert_eq!(selection.is_file_pattern_match("path/to/music.wav"), true);
        assert_eq!(selection.is_file_pattern_match("path/to/music.mp3"), false);
        assert_eq!(selection.is_file_pattern_match("path/to/music.ogg"), false);
        assert_eq!(selection.is_file_pattern_match("path/to/music.aac"), false);

        let selection = Selection::from_patterns(&["*"], &["*.mp3", "*.ogg"], &["*"], &[] as &[&str]).unwrap();

        assert_eq!(selection.is_file_pattern_match("path/to/music.flac"), true);
        assert_eq!(selection.is_file_pattern_match("path/to/music.wav"), true);
        assert_eq!(selection.is_file_pattern_match("path/to/music.aac"), true);
        assert_eq!(selection.is_file_pattern_match("path/to/music.mpc"), true);
        assert_eq!(selection.is_file_pattern_match("path/to/music.mp3"), false);
        assert_eq!(selection.is_file_pattern_match("path/to/music.ogg"), false);

        let selection = Selection::from_patterns(&["*.flac", "*.wav"], &["item*", "self*"], &["*"], &[] as &[&str]).unwrap();

        assert_eq!(selection.is_file_pattern_match("path/to/music.flac"), true);
        assert_eq!(selection.is_file_pattern_match("path/to/music.wav"), true);
        assert_eq!(selection.is_file_pattern_match("path/to/item.flac"), false);
        assert_eq!(selection.is_file_pattern_match("path/to/self.flac"), false);
        assert_eq!(selection.is_file_pattern_match("path/to/music.aac"), false);
        assert_eq!(selection.is_file_pattern_match("path/to/music.mpc"), false);
        assert_eq!(selection.is_file_pattern_match("path/to/music.mp3"), false);
        assert_eq!(selection.is_file_pattern_match("path/to/music.ogg"), false);
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

            let selection = Selection::from_patterns(include_file_patterns, exclude_file_patterns, include_dir_patterns, exclude_dir_patterns).unwrap();
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
                (vec!["music*"], vec!["*.mp3", "*.ogg", "*.aac"], vec!["*"], Vec::<&str>::new(), Sorter::default()),
                vec![
                    path.join("music.flac"),
                    path.join("music.wav"),
                ],
            ),
            (
                (vec!["*.flac"], vec![], vec!["*"], Vec::<&str>::new(), Sorter::default()),
                vec![
                    path.join("item.flac"),
                    path.join("music.flac"),
                    path.join("self.flac"),
                ],
            ),
            (
                (vec!["music*"], vec![], vec!["*"], Vec::<&str>::new(), Sorter::default()),
                vec![
                    path.join("music.aac"),
                    path.join("music.flac"),
                    path.join("music.mp3"),
                    path.join("music.ogg"),
                    path.join("music.wav"),
                ],
            ),
            (
                (vec!["item.*", "self.*"], vec!["*.flac"], vec!["*"], Vec::<&str>::new(), Sorter::default()),
                vec![
                    path.join("item.yml"),
                    path.join("self.yml"),
                ],
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let (include_file_patterns, exclude_file_patterns, include_dir_patterns, exclude_dir_patterns, sort_order) = input;

            let selection = Selection::from_patterns(include_file_patterns, exclude_file_patterns, include_dir_patterns, exclude_dir_patterns).unwrap();
            let produced = selection.select_in_dir_sorted(&path, sort_order).expect("unable to select in dir");
            assert_eq!(expected, produced);
        }
    }
}
