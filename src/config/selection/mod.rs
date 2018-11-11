pub mod matcher;

use std::path::Path;
use std::path::PathBuf;

use config::selection::matcher::Matcher;
use config::sort_order::SortOrder;
use config::selection::matcher::Error as MatcherError;

#[derive(Debug)]
pub enum Error {
    CannotBuildMatcher(MatcherError),
    CannotReadDir(std::io::Error),
    CannotReadDirEntry(std::io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::CannotBuildMatcher(ref err) => write!(f, "cannot build matcher: {}", err),
            Error::CannotReadDir(ref err) => write!(f, "cannot read directory: {}", err),
            Error::CannotReadDirEntry(ref err) => write!(f, "cannot read directory entry: {}", err),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Error::CannotBuildMatcher(ref err) => Some(err),
            Error::CannotReadDir(ref err) => Some(err),
            Error::CannotReadDirEntry(ref err) => Some(err),
        }
    }
}

#[derive(PartialEq, Eq, Hash, Deserialize, Debug)]
#[serde(default)]
pub struct Selection {
    include: Matcher,
    exclude: Matcher,
}

impl Default for Selection {
    fn default() -> Self {
        Selection {
            include: Matcher::any(),
            exclude: Matcher::from_patterns(&["item*", "self*"]).unwrap(),
        }
    }
}

impl Selection {
    pub fn from_patterns<III, SI, IIE, SE>(include_pattern_strs: III, exclude_pattern_strs: IIE) -> Result<Self, Error>
    where
        III: IntoIterator<Item = SI>,
        SI: AsRef<str>,
        IIE: IntoIterator<Item = SE>,
        SE: AsRef<str>,
    {
        let include_selection = Matcher::from_patterns(include_pattern_strs).map_err(Error::CannotBuildMatcher)?;
        let exclude_selection = Matcher::from_patterns(exclude_pattern_strs).map_err(Error::CannotBuildMatcher)?;

        Ok(Self::from_matchers(include_selection, exclude_selection))
    }

    pub fn from_matchers(include: Matcher, exclude: Matcher) -> Self {
        Selection { include, exclude }
    }

    /// Returns true if the path is a pattern match.
    /// In order to be a pattern match, the path must match the include filter, and must NOT match the exclude filter.
    /// This uses only the lexical content of the path, and does not access the filesystem.
    pub fn is_pattern_match<P: AsRef<Path>>(&self, path: P) -> bool {
        self.include.is_match(&path) && !self.exclude.is_match(&path)
    }

    /// Returns true if a path is selected.
    /// Directories are always marked as selected.
    /// Files are selected if they are also a pattern match.
    pub fn is_selected<P: AsRef<Path>>(&self, path: P) -> bool {
        path.as_ref().is_dir() || (path.as_ref().is_file() && self.is_pattern_match(path))
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
        let item_entries = dir_path
            .as_ref()
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
    use super::Error;

    use std::path::Path;

    use serde_yaml;

    #[test]
    fn test_deserialization() {
        // A single pattern for each of include and exclude.
        let text = "include: '*.flac'\nexclude: '*.mp3'";
        let selection: Selection = serde_yaml::from_str(&text).unwrap();

        assert!(selection.is_pattern_match("path/to/music.flac"));
        assert!(!selection.is_pattern_match("path/to/music.mp3"));
        assert!(!selection.is_pattern_match("path/to/music.wav"));

        // Multiple patterns for each of include and exclude.
        let text = "include:\n  - '*.flac'\n  - '*.wav'\nexclude:\n  - '*.mp3'\n  - '*.ogg'";
        let selection: Selection = serde_yaml::from_str(&text).unwrap();

        assert!(selection.is_pattern_match("path/to/music.flac"));
        assert!(selection.is_pattern_match("path/to/music.wav"));
        assert!(!selection.is_pattern_match("path/to/music.mp3"));
        assert!(!selection.is_pattern_match("path/to/music.ogg"));
        assert!(!selection.is_pattern_match("path/to/music.aac"));

        // Using a default value for missing include patterns.
        let text = "exclude:\n  - '*.mp3'\n  - '*.ogg'";
        let selection: Selection = serde_yaml::from_str(&text).unwrap();

        assert!(selection.is_pattern_match("path/to/music.flac"));
        assert!(selection.is_pattern_match("path/to/music.wav"));
        assert!(selection.is_pattern_match("path/to/music.aac"));
        assert!(selection.is_pattern_match("path/to/music.mpc"));
        assert!(!selection.is_pattern_match("path/to/music.mp3"));
        assert!(!selection.is_pattern_match("path/to/music.ogg"));

        // Using a default value for missing exclude patterns.
        let text = "include:\n  - '*.flac'\n  - '*.wav'";
        let selection: Selection = serde_yaml::from_str(&text).unwrap();

        assert!(selection.is_pattern_match("path/to/music.flac"));
        assert!(selection.is_pattern_match("path/to/music.wav"));
        assert!(!selection.is_pattern_match("path/to/item.flac"));
        assert!(!selection.is_pattern_match("path/to/self.flac"));
        assert!(!selection.is_pattern_match("path/to/music.aac"));
        assert!(!selection.is_pattern_match("path/to/music.mpc"));
        assert!(!selection.is_pattern_match("path/to/music.mp3"));
        assert!(!selection.is_pattern_match("path/to/music.ogg"));
    }

    #[test]
    fn test_from_patterns() {
        let passing_inputs = vec![
            Selection::from_patterns(&["*"], &["*"]),
            Selection::from_patterns(&["*.a", "*.b"], &["*.a", "*.b"]),
            Selection::from_patterns(&["?.a", "?.b"], &["?.a", "?.b"]),
            Selection::from_patterns(&["*.a"], &["*.a"]),
            Selection::from_patterns(&["**"], &["**"]),
            Selection::from_patterns(&["a/**/b"], &["a/**/b"]),
            Selection::from_patterns(&[""; 0], &[""; 0]),
            Selection::from_patterns(&[""], &[""]),
            Selection::from_patterns(&["[a-z]*.a"], &["[a-z]*.a"]),
            Selection::from_patterns(&["**", "[a-z]*.a"], &["**", "[a-z]*.a"]),
            Selection::from_patterns(&["[!abc]"], &["[!abc]"]),
            Selection::from_patterns(&["[*]"], &["[*]"]),
            Selection::from_patterns(&["[?]"], &["[?]"]),
            Selection::from_patterns(&["{*.a,*.b,*.c}"], &["{*.a,*.b,*.c}"]),
        ];

        for input in passing_inputs {
            let expected = true;
            let produced = input.is_ok();
            assert_eq!(expected, produced);
        }

        let failing_inputs = vec![
            // Invalid double star
            Selection::from_patterns(&["a**b"], &["a**b"]),

            // Unclosed character class
            Selection::from_patterns(&["[abc"], &["[abc"]),

            // Malformed character range
            Selection::from_patterns(&["[z-a]"], &["[z-a]"]),

            // Unclosed alternates
            Selection::from_patterns(&["{*.a,*.b,*.c"], &["{*.a,*.b,*.c"]),

            // Unopened alternates
            // Selection::from_patterns(&["*.a,*.b,*.c}"], &["*.a,*.b,*.c}"]),

            // Nested alternates
            Selection::from_patterns(&["{*.a,{*.b,*.c}}"], &["{*.a,{*.b,*.c}}"]),

            // Dangling escape
            // Selection::from_patterns(&["*.a\""], &["*.a\""]),
        ];

        for input in failing_inputs {
            match input.unwrap_err() {
                Error::CannotBuildMatcher(_) => {},
                _ => { panic!(); },
            }
        }
    }

    #[test]
    fn test_is_pattern_match() {
        let selection = Selection::from_patterns(&["*.flac"], &["*.mp3"]).unwrap();

        assert!(selection.is_pattern_match("path/to/music.flac"));
        assert!(!selection.is_pattern_match("path/to/music.mp3"));
        assert!(!selection.is_pattern_match("path/to/music.wav"));

        let selection = Selection::from_patterns(&["*.flac", "*.wav"], &["*.mp3", "*.ogg"]).unwrap();

        assert!(selection.is_pattern_match("path/to/music.flac"));
        assert!(selection.is_pattern_match("path/to/music.wav"));
        assert!(!selection.is_pattern_match("path/to/music.mp3"));
        assert!(!selection.is_pattern_match("path/to/music.ogg"));
        assert!(!selection.is_pattern_match("path/to/music.aac"));

        let selection = Selection::from_patterns(&["*"], &["*.mp3", "*.ogg"]).unwrap();

        assert!(selection.is_pattern_match("path/to/music.flac"));
        assert!(selection.is_pattern_match("path/to/music.wav"));
        assert!(selection.is_pattern_match("path/to/music.aac"));
        assert!(selection.is_pattern_match("path/to/music.mpc"));
        assert!(!selection.is_pattern_match("path/to/music.mp3"));
        assert!(!selection.is_pattern_match("path/to/music.ogg"));

        let selection = Selection::from_patterns(&["*.flac", "*.wav"], &["item*", "self*"]).unwrap();

        assert!(selection.is_pattern_match("path/to/music.flac"));
        assert!(selection.is_pattern_match("path/to/music.wav"));
        assert!(!selection.is_pattern_match("path/to/item.flac"));
        assert!(!selection.is_pattern_match("path/to/self.flac"));
        assert!(!selection.is_pattern_match("path/to/music.aac"));
        assert!(!selection.is_pattern_match("path/to/music.mpc"));
        assert!(!selection.is_pattern_match("path/to/music.mp3"));
        assert!(!selection.is_pattern_match("path/to/music.ogg"));
    }
}
