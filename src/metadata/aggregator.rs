
use metadata::processor::MetaProcessor;
use metadata::processor::Error as ProcessorError;

#[derive(Debug)]
pub enum Error {
    CannotProcessMetadata(ProcessorError),
    CannotSelectPaths(SelectionError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::CannotProcessMetadata(ref err) => write!(f, "cannot process metadata: {}", err),
            Error::CannotSelectPaths(ref err) => write!(f, "cannot select item paths: {}", err),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Error::CannotProcessMetadata(ref err) => Some(err),
            Error::CannotSelectPaths(ref err) => Some(err),
        }
    }
}

/// Different ways to process child metadata into desired outputs.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AggMethod {
    Collect,
    First,
}

use std::path::Path;
use std::collections::BTreeMap;
use std::collections::VecDeque;

use library::selection::Selection;
use library::selection::Error as SelectionError;
use library::sort_order::SortOrder;
use metadata::reader::MetaFormat;
use metadata::types::MetaVal;

pub struct MetaAggregator;

impl MetaAggregator {
    pub fn resolve_field<P, S>(
        item_path: P,
        field: S,
        meta_format: MetaFormat,
        selection: &Selection,
        sort_order: SortOrder,
    ) -> Result<Option<MetaVal>, Error>
    where
        P: AsRef<Path>,
        S: AsRef<str>,
    {
        let mut mb = MetaProcessor::process_item_file_flattened(item_path, meta_format, selection, sort_order).map_err(Error::CannotProcessMetadata)?;
        Ok(mb.remove(field.as_ref()))
    }

    pub fn resolve_field_children_helper<P, S>(
        item_path: P,
        field: S,
        meta_format: MetaFormat,
        selection: &Selection,
        sort_order: SortOrder,
        agg_methods: &BTreeMap<String, AggMethod>,
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
            let sub_item_paths = selection.select_in_dir(frontier_item_path).map_err(Error::CannotSelectPaths)?;
            for sub_item_path in sub_item_paths {
                match Self::resolve_field(&sub_item_path, &field, meta_format, &selection, sort_order)? {
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
