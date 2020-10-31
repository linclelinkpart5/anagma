use std::borrow::Cow;
use std::ffi::OsString;
use std::io::{Error as IoError, ErrorKind as IoErrorKind, Result as IoResult};
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::config::selection::Selection;
use crate::metadata::schema::SchemaFormat;
use crate::util::NameError;
use crate::util::Util;

#[derive(Debug, Error)]
pub enum SourceError {
    #[error("source file name is invalid: {0}")]
    InvalidName(String),
    #[error("source file name does not have an extension: {0}")]
    EmptyExtension(String),
    #[error("unknown format for file extension: {}", .0.to_string_lossy())]
    UnknownExtension(OsString),
}

#[derive(Debug, Error)]
pub enum LookupError {
    #[error("not a directory: {}", .0.display())]
    NotADir(PathBuf),
    #[error("not a file: {}", .0.display())]
    NotAFile(PathBuf),

    #[error(r#"cannot access item path "{}": {1}"#, .0.display())]
    ItemAccess(PathBuf, #[source] IoError),
    #[error(r#"cannot access meta path "{}": {1}"#, .0.display())]
    MetaAccess(PathBuf, #[source] IoError),

    #[error("item path does not have a parent: {}", .0.display())]
    NoItemParentDir(PathBuf),
    #[error("meta path does not have a parent: {}", .0.display())]
    NoMetaParentDir(PathBuf),

    #[error("unable to read item directory: {0}")]
    IterDir(#[source] IoError),
    #[error("unable to read item directory entry: {0}")]
    IterDirEntry(#[source] IoError),
    // Bulk(IoError, Vec<IoError>),
}

impl LookupError {
    pub(crate) fn is_fatal(&self) -> bool {
        match self {
            Self::MetaAccess(_, io_error) => match io_error.kind() {
                IoErrorKind::NotFound => false,
                _ => true,
            },
            Self::NotADir(..) | Self::NoItemParentDir(..) => false,
            _ => true,
        }
    }
}

/// Represents a method of finding the location of a meta file given an item
/// file path.
#[derive(Clone, Copy)]
pub enum Anchor {
    /// The meta file is located in the same directory as the item file path.
    External,

    /// The meta file is located inside the item file path.
    /// Implies that the the item file path is a directory.
    Internal,
}

enum NS {
    Name(String),
    Stub(String, SchemaFormat),
}

/// Defines a meta file source, consisting of an anchor (the target directory
/// to look in) and a file name (the meta file name in that target directory).
pub struct Source {
    name: String,
    anchor: Anchor,
}

impl Source {
    fn _new(ns: NS, anchor: Anchor) -> Result<Self, NameError> {
        let (atom, opt_format) = match ns {
            NS::Name(name) => (name, None),
            NS::Stub(stub, fmt) => (stub, Some(fmt)),
        };

        let mut validated = Util::validate_item_name(atom)?;

        if let Some(format) = opt_format {
            let ext = format.file_extension();
            validated.reserve(ext.len() + 1);
            validated.push('.');
            validated.push_str(ext);
        }

        Ok(Self {
            name: validated,
            anchor,
        })
    }

    pub fn from_name(name: String, anchor: Anchor) -> Result<Self, NameError> {
        Self::_new(NS::Name(name), anchor)
    }

    pub fn from_stub(
        stub: String,
        format: SchemaFormat,
        anchor: Anchor,
    ) -> Result<Self, NameError> {
        Self::_new(NS::Stub(stub, format), anchor)
    }

    /// Given a concrete item file path, returns the meta file path that would
    /// provide metadata for that item path, according to the source rules.
    pub fn meta_path(&self, item_path: &Path) -> Result<PathBuf, LookupError> {
        // Get filesystem stat for item path.
        // This step is always done, even if the file/directory status does not
        // need to be checked, as it provides useful error information about
        // permissions and non-existence.
        let item_fs_stat = std::fs::metadata(&item_path)
            .map_err(|io| LookupError::ItemAccess(item_path.into(), io))?;

        let meta_path_parent_dir = match self.anchor {
            Anchor::External => item_path
                .parent()
                .ok_or_else(|| LookupError::NoItemParentDir(item_path.into()))?,
            Anchor::Internal => {
                if !item_fs_stat.is_dir() {
                    return Err(LookupError::NotADir(item_path.into()));
                }

                item_path
            }
        };

        // Create the target meta file path.
        let meta_path = meta_path_parent_dir.join(&self.name);

        // Get filesystem stat for meta path.
        // NOTE: Using `match` in order to avoid a clone in the error case.
        // TODO: Move fatal error determination logic here.
        let meta_fs_stat = match std::fs::metadata(&meta_path) {
            Ok(o) => o,
            Err(io_err) => return Err(LookupError::MetaAccess(meta_path, io_err)),
        };

        // Ensure that the meta path is indeed a file.
        if !meta_fs_stat.is_file() {
            // Found a directory with the meta file name, this would be an unusual error case.
            Err(LookupError::NotAFile(meta_path))
        } else {
            Ok(meta_path)
        }
    }

    /// Provides a listing of the item file paths that this meta target
    /// could/should provide metadata for. Note that this does NOT parse meta
    /// files, it only uses file system locations and presence. In addition, no
    /// filtering or sorting of the returned item paths is performed.
    pub fn item_paths<'a>(&self, meta_path: &'a Path) -> Result<ItemPaths<'a>, LookupError> {
        let meta_fs_stat = std::fs::metadata(&meta_path)
            .map_err(|io| LookupError::MetaAccess(meta_path.into(), io))?;

        if !meta_fs_stat.is_file() {
            return Err(LookupError::NotAFile(meta_path.into()));
        }

        // Get the parent directory of the meta file.
        if let Some(meta_parent_dir_path) = meta_path.parent() {
            let ipi = match self.anchor {
                Anchor::External => {
                    // Return all children of the parent directory of this meta file.
                    let read_dir =
                        std::fs::read_dir(&meta_parent_dir_path).map_err(LookupError::IterDir)?;

                    ItemPathsInner::ReadDir(read_dir)
                }
                Anchor::Internal => {
                    // This is just the passed-in path, just push it on unchanged.
                    ItemPathsInner::Single(Some(meta_parent_dir_path))
                }
            };

            Ok(ItemPaths(ipi))
        } else {
            // This should never happen, since at this point we have a real meta
            // file and thus, a real parent directory for that file, but making
            // an error for it anyways.
            Err(LookupError::NoMetaParentDir(meta_path.into()))
        }
    }

    /// Similar to `item_paths`, but also performs selection filtering on the
    /// produced item paths.
    pub fn selected_item_paths<'a>(
        &self,
        meta_path: &'a Path,
        selection: &'a Selection,
    ) -> Result<SelectedItemPaths<'a>, LookupError> {
        Ok(SelectedItemPaths(self.item_paths(meta_path)?, selection))
    }
}

// pub struct OldSourcer(Vec<Source>, SchemaFormat);

// impl OldSourcer {
//     pub fn new(sources: Vec<Source>, format: SchemaFormat) -> Self {
//         Self(sources, format)
//     }

//     pub fn from_stubs(stubs: Vec<String>, format: SchemaFormat) -> Result<Self, NameError> {
//         for stub in stubs {
//             // Ensure that the stub is a valid file name.
//             let mut stub = Util::validate_item_name(stub)?;

//             // It is expected that appending a suffix at the end will still keep
//             // a valid file name.
//             stub.push('.');
//             stub.push_str(format.file_extension());
//         }

//         let mut sources = Vec::new();

//         Ok(Self(sources, format))
//     }

//     pub fn from_sources(sources: Vec<Source>) -> Result<Self, ()> {
//         // Try and automatically detect format by looking at the sources.
//         // All sources must have either no extension or a known format
//         // extension, at least one source must have a known format extension,
//         // and all sources that have an extension must have the same extension.

//         todo!()
//     }
// }

// Only strings, the schema format is only used to convert file stubs into
// file names. Since the responsibility of this type is only to produce
// candidate meta paths, and not actually parse them, this does not need to
// store schema formats.
pub struct Sourcer(Vec<Source>);

impl Sourcer {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn source(&mut self, source: Source) -> &mut Self {
        self.0.push(source);
        self
    }
}

pub struct SourcerMetaPaths<'a> {
    iter: std::slice::Iter<'a, Source>,
    item_path: &'a Path,
}

impl<'a> Iterator for SourcerMetaPaths<'a> {
    type Item = Result<PathBuf, LookupError>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(source) = self.iter.next() {
            let res = source.meta_path(self.item_path);

            match res {
                Ok(meta_path) => {
                    return Some(Ok(meta_path));
                }
                Err(err) if err.is_fatal() => {
                    return Some(Err(err));
                }
                Err(_) => {
                    continue;
                }
            }
        }

        None
    }
}

pub struct SourcerItemPaths<'a> {
    source_iter: std::slice::Iter<'a, Source>,
    meta_path: &'a Path,
    curr_item_path_iter: Option<ItemPaths<'a>>,
}

impl<'a> Iterator for SourcerItemPaths<'a> {
    type Item = Result<Cow<'a, Path>, LookupError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // If there is no current item path iterator, pull the next source
            // and create one.
            if self.curr_item_path_iter.is_none() {
                // If there are no more sources, this entire iterator is done.
                let src = self.source_iter.next()?;
                let it = match src.item_paths(&self.meta_path) {
                    Err(err) => {
                        return Some(Err(err));
                    }
                    Ok(it) => it,
                };

                self.curr_item_path_iter = Some(it);
            }

            let item_path_iter = self.curr_item_path_iter.as_mut()?;

            match item_path_iter.next() {
                Some(Err(io_err)) => {
                    return Some(Err(LookupError::IterDirEntry(io_err)));
                },
                Some(Ok(item_path)) => {
                    return Some(Ok(item_path));
                },
                None => {
                    // This item path iter has been exhausted, drop it and
                    // restart the loop.
                    self.curr_item_path_iter = None;
                    continue;
                },
            }
        }
    }
}

enum ItemPathsInner<'a> {
    ReadDir(std::fs::ReadDir),
    Single(Option<&'a Path>),
}

impl<'a> Iterator for ItemPathsInner<'a> {
    type Item = IoResult<Cow<'a, Path>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::ReadDir(rd) => Some(rd.next()?.map(|e| Cow::Owned(e.path()))),
            Self::Single(o) => o.take().map(|p| Ok(Cow::Borrowed(p))),
        }
    }
}

pub struct ItemPaths<'a>(ItemPathsInner<'a>);

impl<'a> Iterator for ItemPaths<'a> {
    type Item = IoResult<Cow<'a, Path>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

pub struct SelectedItemPaths<'a>(ItemPaths<'a>, &'a Selection);

impl<'a> Iterator for SelectedItemPaths<'a> {
    type Item = IoResult<Cow<'a, Path>>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(res) = self.0.next() {
            match res {
                Err(err) => {
                    return Some(Err(err));
                }
                Ok(path) => match self.1.is_selected(&path) {
                    Ok(true) => {
                        return Some(Ok(path));
                    }
                    Ok(false) => {
                        continue;
                    }
                    Err(err) => {
                        return Some(Err(err));
                    }
                },
            }
        }

        None
    }
}
