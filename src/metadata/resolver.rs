//! Manages field-based lookups of metadata.

use std::fmt::Display;
use std::fmt::Result as FmtResult;
use std::fmt::Formatter;

use failure::Backtrace;
use failure::Context;
use failure::Fail;
use failure::ResultExt;

#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Fail, Hash)]
#[non_exhaustive]
pub enum ErrorKind {
    #[fail(display = "cannot process metadata file")]
    CannotProcessMetaFile,
    #[fail(display = "cannot read files in directory")]
    CannotReadDirFiles,
}

impl Fail for Error {
    fn cause(&self) -> Option<&Fail> { self.inner.cause() }
    fn backtrace(&self) -> Option<&Backtrace> { self.inner.backtrace() }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult { Display::fmt(&self.inner, f) }
}

impl Error {
    pub fn kind(&self) -> &ErrorKind { self.inner.get_context() }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error { Error { inner: Context::new(kind) } }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Error { Error { inner: inner } }
}

use std::path::Path;
use std::marker::PhantomData;
use std::collections::VecDeque;

use library::config::Config;
use metadata::types::MetaVal;
use metadata::processor::MetaProcessor;
use metadata::reader::MetaReader;
use metadata::location::MetaLocation;

const LOCATION_LIST: &[MetaLocation] = &[MetaLocation::Siblings, MetaLocation::Contains];

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AggKind {
    Seq,
}

pub struct MetaResolver<MR>(PhantomData<MR>);

impl<MR> MetaResolver<MR>
where
    MR: MetaReader,
{
    pub fn resolve_field<P, S>(
        item_path: P,
        field: S,
        config: &Config,
    ) -> Result<Option<MetaVal>, Error>
    where
        P: AsRef<Path>,
        S: AsRef<str>,
    {
        let mut mb = MetaProcessor::<MR>::process_item_file_flattened(
            item_path,
            LOCATION_LIST.to_vec(),
            &config,
        ).context(ErrorKind::CannotProcessMetaFile)?;

        Ok(mb.remove(field.as_ref()))
    }

    pub fn resolve_field_parents<P, S>(
        item_path: P,
        field: S,
        config: &Config,
    ) -> Result<Option<MetaVal>, Error>
    where
        P: AsRef<Path>,
        S: AsRef<str>,
    {
        // LEARN: The first item in `.ancestors()` is the original path, so it needs to be skipped.
        for ancestor_item_path in item_path.as_ref().ancestors().into_iter().skip(1) {
            let opt_val = Self::resolve_field(&item_path, &field, &config)?;

            if opt_val.is_some() {
                return Ok(opt_val)
            }
        }

        Ok(None)
    }

    fn resolve_field_children_helper<P, S>(
        item_path: P,
        field: S,
        config: &Config,
    ) -> Result<Vec<MetaVal>, Error>
    where
        P: AsRef<Path>,
        S: AsRef<str>,
    {
        let item_path = item_path.as_ref();

        let mut frontier = VecDeque::new();

        if item_path.is_dir() {
            frontier.push_back(item_path.to_owned());
        }

        let mut child_results = vec![];

        // For each path in the frontier, look at the items contained within it.
        // Assume that the paths in the frontier are directories.
        while let Some(frontier_item_path) = frontier.pop_front() {
            // Get sub items contained within.
            let sub_item_paths = config.select_in_dir(frontier_item_path).context(ErrorKind::CannotReadDirFiles)?;

            for sub_item_path in sub_item_paths {
                match Self::resolve_field(&sub_item_path, &field, &config)? {
                    Some(sub_meta_val) => {
                        child_results.push(sub_meta_val);
                    },
                    None => {
                        // If the sub item is a directory, add it to the frontier.
                        if sub_item_path.is_dir() {
                            // Since a depth-first search is desired, treat as a stack.
                            frontier.push_front(sub_item_path);
                        }
                    },
                }
            }
        }

        Ok(child_results)
    }
}

// #[cfg(test)]
// mod tests {
//     use super::MetaResolver;

//     use library::config::Config;
//     use metadata::reader::yaml::YamlMetaReader;

//     use test_util::create_temp_media_test_dir;

//     #[test]
//     fn test_resolve_field_children_helper() {
//         use std::time::Duration;
//         use std::thread::sleep;

//         let temp_dir = create_temp_media_test_dir("test_resolve_field_children_helper");

//         let path = temp_dir.path();
//         let field = "TRACK_01_item_key";
//         let config = Config::default();

//         let result = MetaResolver::<YamlMetaReader>::resolve_field_children_helper(&path, &field, &config).unwrap();

//         println!("{:?}", result);

//         // let result = MetaProcessor::process_meta_file::<YamlMetaReader, _>(path.join("ALBUM_01").join("item.yml"), MetaLocation::Contains, &config);

//         // println!("{:?}", result);
//     }
// }
