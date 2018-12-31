mod inherit;
mod collect;

use std::path::Path;
use std::path::PathBuf;
use std::collections::VecDeque;
use std::collections::HashMap;

use config::meta_format::MetaFormat;
use config::selection::Selection;
use config::sort_order::SortOrder;
use metadata::types::MetaBlock;
use metadata::types::MetaKey;
use metadata::processor::MetaProcessor;

use self::inherit::InheritMethod;
use self::collect::CollectMethod;


pub type FallbackSpec = HashMap<String, FallbackSpecNode>;

/// Node type for the tree representation of fallback methods.
pub enum FallbackSpecNode {
    Children(HashMap<MetaKey, FallbackSpecNode>),
    Method(FallbackMethod),
}

/// Different ways to process parent metadata into desired outputs.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FallbackMethod {
    Inherit(InheritMethod),
    Collect(CollectMethod),
}

struct CollectIterator<'s> {
    frontier: VecDeque<PathBuf>,
    last_processed_path: Option<PathBuf>,

    meta_format: MetaFormat,
    selection: &'s Selection,
    sort_order: SortOrder,
}

impl<'s> CollectIterator<'s> {
    pub fn new<P: AsRef<Path>>(
        start_item_path: P,
        meta_format: MetaFormat,
        selection: &'s Selection,
        sort_order: SortOrder,
    ) -> Self
    {
        // Initialize the frontier with the subitems of the start item path.
        let mut frontier = VecDeque::new();

        match selection.select_in_dir_sorted(start_item_path, sort_order) {
            Err(err) => {
                warn!("{}", err);
            },
            Ok(mut sub_item_paths) => {
                for p in sub_item_paths.drain(..) {
                    frontier.push_back(p);
                }
            },
        };

        CollectIterator {
            frontier,
            last_processed_path: None,
            meta_format,
            selection,
            sort_order,
        }
    }

    /// Manually delves into a directory, and adds its subitems to the frontier.
    pub fn delve(&mut self) -> Option<PathBuf> {
        if let Some(lpp) = self.last_processed_path.take() {
            // If the last processed path is a directory, add its children to the frontier.
            if lpp.is_dir() {
                match self.selection.select_in_dir_sorted(&lpp, self.sort_order) {
                    Err(err) => {
                        warn!("{}", err);
                    },
                    Ok(mut sub_item_paths) => {
                        for p in sub_item_paths.drain(..).rev() {
                            self.frontier.push_front(p);
                        }
                    },
                }
            }

            Some(lpp)
        }
        else {
            None
        }
    }
}

impl<'p, 's> Iterator for CollectIterator<'p> {
    type Item = MetaBlock;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(frontier_item_path) = self.frontier.pop_front() {
            let ret_mb = match MetaProcessor::process_item_file(
                &frontier_item_path,
                self.meta_format,
                self.selection,
                self.sort_order,
            ) {
                Ok(mb) => Some(mb),
                Err(err) => {
                    warn!("{}", err);
                    self.next()
                },
            };

            // Save the most recently processed item path.
            self.last_processed_path = Some(frontier_item_path);

            ret_mb
        }
        else {
            None
        }
    }
}

// pub fn does_this_work() {
//     let mut ci = CollectIterator {
//         frontier: VecDeque::new(),
//         last_processed_path: None,

//         meta_format: MetaFormat::default(),
//         selection: &Selection::default(),
//         sort_order: SortOrder::default(),
//     };

//     while let Some(_x) = ci.next() {
//         ci.delve();
//     }
// }
