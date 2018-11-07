
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
use std::collections::VecDeque;

use library::selection::Selection;
use library::selection::Error as SelectionError;
use library::sort_order::SortOrder;
use metadata::reader::MetaFormat;
use metadata::types::MetaVal;
use util::GenConverter;

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
        let mut mb = MetaProcessor::process_item_file_flattened(
            item_path,
            meta_format,
            selection,
            sort_order,
        ).map_err(Error::CannotProcessMetadata)?;

        Ok(mb.remove(field.as_ref()))
    }

    pub fn resolve_field_children<P, S>(
        item_path: P,
        field: S,
        meta_format: MetaFormat,
        selection: &Selection,
        sort_order: SortOrder,
        agg_method: AggMethod,
    ) -> Result<MetaVal, Error>
    where
        P: AsRef<Path>,
        S: AsRef<str>,
    {
        // This iterates over and unwraps `Ok` values, while also logging `Err` values.
        let mut gen = Self::resolve_field_children_helper(item_path, field, meta_format, selection, sort_order)
            .filter_map(|res| match res {
                Ok(mv) => Some(mv),
                Err(err) => {
                    warn!("{}", err);
                    None
                },
            });

        let ret_mv = match agg_method {
            AggMethod::First => {
                // Get the first item from the generator.
                match gen.next() {
                    Some(mv) => mv,
                    None => MetaVal::Nil,
                }
            },
            AggMethod::Collect => {
                // Collect all items from the generator.
                let mvs = gen.collect::<Vec<_>>();

                MetaVal::Seq(mvs)
            },
        };

        Ok(ret_mv)
    }

    pub fn resolve_field_children_helper<'a, P, S>(
        item_path: P,
        field: S,
        meta_format: MetaFormat,
        selection: &'a Selection,
        sort_order: SortOrder,
    ) -> impl Iterator<Item = Result<MetaVal, Error>> + 'a
    where
        P: AsRef<Path>,
        S: AsRef<str> + 'a,
    {
        let item_path = item_path.as_ref();
        let mut frontier = VecDeque::new();
        if item_path.is_dir() {
            frontier.push_back(item_path.to_owned());
        }

        let closure = move || {
            // For each path in the frontier, look at the items contained within it.
            // Assume that the paths in the frontier are directories.
            while let Some(frontier_item_path) = frontier.pop_front() {
                // Get sub items contained within.
                match selection.select_in_dir(frontier_item_path).map_err(Error::CannotSelectPaths) {
                    Err(err) => {
                        yield Err(err);
                        continue;
                    },
                    Ok(sub_item_paths) => {
                        for sub_item_path in sub_item_paths {
                            match Self::resolve_field(&sub_item_path, &field, meta_format, &selection, sort_order) {
                                Err(err) => {
                                    yield Err(err);
                                    continue;
                                },
                                Ok(Some(sub_meta_val)) => {
                                    yield Ok(sub_meta_val);
                                },
                                Ok(None) => {
                                    // If the sub item is a directory, add it to the frontier.
                                    if sub_item_path.is_dir() {
                                        // Since a depth-first search is desired, treat as a stack.
                                        frontier.push_front(sub_item_path);
                                    }
                                },
                            }
                        }
                    },
                }
            }
        };


        GenConverter::gen_to_iter(closure)
    }
}

#[cfg(test)]
mod tests {
    use super::MetaAggregator;

    use library::config::Config;
    use library::sort_order::SortOrder;
    use metadata::reader::MetaFormat;
    use metadata::location::MetaLocation;
    use metadata::types::MetaVal;

    use test_util::create_temp_media_test_dir;

    #[test]
    fn test_resolve_field_children_helper() {
        let temp_dir = create_temp_media_test_dir("test_resolve_field_children_helper");
        let path = temp_dir.path();

        let config = Config::default();
        let selection = &config.selection;

        let gen = MetaAggregator::resolve_field_children_helper(path, "TRACK_01_item_key", MetaFormat::Yaml, selection, SortOrder::Name);

        for mv in gen {
            println!("{:?}", mv.unwrap());
        }
    }
}

