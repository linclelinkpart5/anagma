//! Manages field-based lookups of metadata.

use std::path::Path;
use std::marker::PhantomData;
use std::collections::VecDeque;

use failure::Error;

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

    fn resolve_field_children_helper<P, S, C>(
        item_path: P,
        field: S,
        config: C,
    ) -> Result<Vec<Option<MetaVal>>, Error>
    where
        P: AsRef<Path>,
        S: AsRef<str>,
        C: AsRef<Config>,
    {
        let item_path = item_path.as_ref();

        // For breadth-first search.
        let mut frontier: VecDeque<&Path> = VecDeque::new();

        frontier.push_back(item_path);

        let mut child_results = vec![];

        // For each path in the frontier, look at the items contained within it.
        // Assume that the paths in the frontier are directories.
        while let Some(frontier_item_path) = frontier.pop_front() {
            // Get sub items contained within.
            let sub_item_paths = config.as_ref().select_in_dir(frontier_item_path)?;

            for sub_item_path in sub_item_paths {
                match Self::resolve_field(&sub_item_path, &field, config.as_ref())? {
                    Some(sub_meta_val) => {
                        child_results.push(Some(sub_meta_val));
                    },
                    None => {},
                }
            }
        }

        Ok(child_results)

        // let sub_item_paths = config.as_ref().select_in_dir(item_path)?;

        // let closure = move || {
        //     for sub_item_path in sub_item_paths {
        //         let child_result = Self::resolve_field(&sub_item_path, &field, config.as_ref());

        //         match child_result {
        //             Ok(Some(_)) => {
        //                 // Found a value, emit it.
        //                 yield child_result;
        //             },
        //             Err(_) => {
        //                 // Found an error, emit it.
        //                 yield child_result;
        //             },
        //             Ok(None) => {
        //                 // In this case, emit the results of a recursive call with this new sub path.
        //                 let t = Box::new(Self::resolve_field_children_helper(&sub_item_path, &field, &config));
        //                 match t {
        //                     Ok(yielder) => {
        //                         for r in yielder {
        //                             yield r;
        //                         }
        //                     },
        //                     Err(e) => {
        //                         yield Err(e);
        //                     },
        //                 }
        //             },
        //         };
        //     }

        //     yield Ok(None)
        // };

        // Ok(GenConverter::gen_to_iter(closure))

        // // Check if the item path is a directory.
        // match item_path.is_dir() {
        //     false => Ok(None),
        //     true => {
        //         let sub_item_paths = config.select_in_dir(item_path)?;

        //         for sub_item_path in sub_item_paths {
        //             let opt_child_result = Self::resolve_field(&sub_item_path, &field, &config)?;

        //             if let Some(child_result) = opt_child_result {
        //                 this_results.push(child_result);
        //             }
        //             else {
        //                 // Recurse!
        //                 Self::resolve_field_children_helper(&sub_item_path, &field, &config)?;
        //             }
        //         }

        //         Ok(Some(vec![]))
        //     },
        // }
    }
}
