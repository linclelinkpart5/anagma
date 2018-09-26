use std::path::Path;

use globset::Glob;
use globset::GlobSet;
use globset::GlobSetBuilder;
use failure::Error;

pub struct Selection(GlobSet);

impl Selection {
    pub fn from_patterns<II, S>(patterns: II) -> Result<Self, Error>
    where
        II: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut builder = GlobSetBuilder::new();

        for pattern in patterns.into_iter() {
            builder.add(Glob::new(pattern.as_ref())?);
        }

        let selection = builder.build()?;

        Ok(Selection(selection))
    }

    pub fn is_match<P: AsRef<Path>>(&self, path: P) -> bool {
        self.0.is_match(path.as_ref())
    }
}

impl Default for Selection {
    fn default() -> Self {
        Selection::from_patterns(&["*"]).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::Selection;

    use std::path::Path;

    #[test]
    fn test_is_match() {
        let selection_a = Selection::from_patterns(&["*.a", "*.b"]).unwrap();
        let selection_b = Selection::from_patterns(&["*.b"]).unwrap();
        let selection_c = Selection::from_patterns(&["*.a", "*.c"]).unwrap();
        let selection_d = Selection::from_patterns(&["*"]).unwrap();

        assert_eq!(selection_a.is_match(Path::new("path.a")), true);
        assert_eq!(selection_a.is_match(Path::new("path.b")), true);
        assert_eq!(selection_a.is_match(Path::new("path.c")), false);
        assert_eq!(selection_a.is_match(Path::new("path.ab")), false);
        assert_eq!(selection_a.is_match(Path::new("path")), false);

        assert_eq!(selection_b.is_match(Path::new("path.a")), false);
        assert_eq!(selection_b.is_match(Path::new("path.b")), true);
        assert_eq!(selection_b.is_match(Path::new("path.c")), false);
        assert_eq!(selection_b.is_match(Path::new("path.ab")), false);
        assert_eq!(selection_b.is_match(Path::new("path")), false);

        assert_eq!(selection_c.is_match(Path::new("path.a")), true);
        assert_eq!(selection_c.is_match(Path::new("path.b")), false);
        assert_eq!(selection_c.is_match(Path::new("path.c")), true);
        assert_eq!(selection_c.is_match(Path::new("path.ab")), false);
        assert_eq!(selection_c.is_match(Path::new("path")), false);

        assert_eq!(selection_d.is_match(Path::new("path.a")), true);
        assert_eq!(selection_d.is_match(Path::new("path.b")), true);
        assert_eq!(selection_d.is_match(Path::new("path.c")), true);
        assert_eq!(selection_d.is_match(Path::new("path.ab")), true);
        assert_eq!(selection_d.is_match(Path::new("path")), true);
    }

    #[test]
    fn test_default() {
        // The default value should be a "match-any" pattern.
        let selection = Selection::default();

        assert_eq!(selection.is_match(Path::new("path")), true);
        assert_eq!(selection.is_match(Path::new("path.a")), true);
        assert_eq!(selection.is_match(Path::new("path.a.b.c")), true);
        assert_eq!(selection.is_match(Path::new("path.ab")), true);
        assert_eq!(selection.is_match(Path::new("")), true);
    }
}
