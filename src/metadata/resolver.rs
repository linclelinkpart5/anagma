//! Manages field-based lookups of metadata.

use std::path::Path;
use std::marker::PhantomData;
use std::collections::VecDeque;

use failure::Error;

use error::ErrorKind;
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
        let mut mb = MetaProcessor::<MR>::composite_item_file(
            item_path,
            LOCATION_LIST.to_vec(),
            &config,
        )?;

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
    ) -> Result<Vec<Option<MetaVal>>, Error>
    where
        P: AsRef<Path>,
        S: AsRef<str>,
    {
        let item_path = item_path.as_ref();

        let mut frontier = VecDeque::new();

        frontier.push_back(item_path.to_owned());

        let mut child_results = vec![];

        // For each path in the frontier, look at the items contained within it.
        // Assume that the paths in the frontier are directories.
        while let Some(frontier_item_path) = frontier.pop_front() {
            // Get sub items contained within.
            match config.select_in_dir(frontier_item_path) {
                Ok(sub_item_paths) => {
                    // Sub paths were found.
                    for sub_item_path in sub_item_paths {
                        match Self::resolve_field(&sub_item_path, &field, &config)? {
                            Some(sub_meta_val) => {
                                child_results.push(Some(sub_meta_val));
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
                },
                Err(err) => {
                    // Unable to look inside frontier item, possible for it to not be a directory.
                    match err.downcast_ref::<ErrorKind>() {
                        // A non-directory was found in the frontier, just skip it.
                        Some(ErrorKind::CannotReadDir(_)) => {},

                        // Any other error is a raise-able offense.
                        _ => Err(err)?,
                    }
                },
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
