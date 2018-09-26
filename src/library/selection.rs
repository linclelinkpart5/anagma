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
