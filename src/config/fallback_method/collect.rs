use std::path::Path;
use std::collections::HashMap;

use metadata::types::MetaVal;
use metadata::types::MetaBlock;
use config::meta_format::MetaFormat;
use config::selection::Selection;
use config::selection::Error as SelectionError;
use config::sort_order::SortOrder;
use metadata::processor::MetaProcessor;

use std::path::PathBuf;
use std::collections::VecDeque;

//     pub fn resolve_field_children_helper<'a, P, S>(
//         item_path: P,
//         field: S,
//         meta_format: MetaFormat,
//         selection: &'a Selection,
//         sort_order: SortOrder,
//     ) -> impl Iterator<Item = Result<(MetaVal, PathBuf), Error>> + 'a
//     where
//         P: AsRef<Path>,
//         S: AsRef<str> + 'a,
//     {
//         let item_path = item_path.as_ref();
//         let mut frontier = VecDeque::new();
//         if item_path.is_dir() {
//             frontier.push_back(item_path.to_owned());
//         }

//         let closure = move || {
//             // Process the initial potential item in the frontier.
//             // LEARN: This awkward step is needed due to lifetime/generator issues and wanting to have errors in the generator.
//             // TODO: Maybe OK to have an error outside of the generator?
//             if let Some(start_item_path) = frontier.pop_front() {
//                 match selection.select_in_dir_sorted(start_item_path, sort_order).map_err(Error::CannotSelectPaths) {
//                     Err(err) => {
//                         yield Err(err);
//                     },
//                     Ok(mut sub_item_paths) => {
//                         for p in sub_item_paths.drain(..) {
//                             frontier.push_back(p);
//                         }
//                     },
//                 }
//             }

//             // For each path in the frontier, look at the items contained within it.
//             while let Some(frontier_item_path) = frontier.pop_front() {
//                 match Self::resolve_field(&frontier_item_path, &field, meta_format, &selection, sort_order) {
//                     Err(err) => {
//                         yield Err(err);
//                     },
//                     Ok(Some(sub_meta_val)) => {
//                         yield Ok((sub_meta_val, frontier_item_path));
//                     },
//                     Ok(None) => {
//                         // If the sub item is a directory, add its children to the frontier.
//                         if frontier_item_path.is_dir() {
//                             match selection.select_in_dir_sorted(frontier_item_path, sort_order).map_err(Error::CannotSelectPaths) {
//                                 Err(err) => {
//                                     yield Err(err);
//                                 },
//                                 Ok(mut sub_item_paths) => {
//                                     for p in sub_item_paths.drain(..).rev() {
//                                         frontier.push_front(p);
//                                     }
//                                 },
//                             }
//                         }
//                     }
//                 }
//             }
//         };
//         GenConverter::gen_to_iter(closure)
//     }

/// Different ways to process child metadata into desired outputs.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CollectMethod {
    Iterate,
    First,
}

impl CollectMethod {
    pub fn process<P: AsRef<Path>>(
        start_item_path: P,
        method_map: &HashMap<String, CollectMethod>,
        meta_format: MetaFormat,
        selection: &Selection,
        sort_order: SortOrder,
    ) -> MetaBlock {
        let mut composed_result_mb = MetaBlock::new();

        let mut frontier = VecDeque::new();

        // The metadata of the starting item is not considered.
        // Add the children of the starting item to the frontier.
        match selection.select_in_dir_sorted(start_item_path, sort_order) {
            Ok(mut sub_item_paths) => {
                for p in sub_item_paths.drain(..) {
                    frontier.push_back(p);
                }
            },
            Err(err) => {
                // If the error is that the item is not a directory, continue gracefully.
                // Otherwise, warn.
                match err {
                    SelectionError::InvalidDirPath(..) => {},
                    _ => { warn!("{}", err); },
                }
            },
        }

        // For each path in the frontier, load its metadata.
        while let Some(frontier_item_path) = frontier.pop_front() {
            match MetaProcessor::process_item_file(
                frontier_item_path,
                meta_format,
                selection,
                sort_order,
            ) {
                Ok(mb) => {

                },
                Err(err) => {
                    warn!("{}", err);
                },
            }
        }

        composed_result_mb
    }
}

#[cfg(test)]
mod tests {
    use metadata::types::MetaVal;

    use super::CollectMethod;

    // #[test]
    // fn test_process() {
    //     let inputs_and_expected = vec![
    //         (
    //             (
    //                 CollectMethod::First,
    //                 vec![
    //                     MetaVal::Str(String::from("A")),
    //                 ],
    //             ),
    //             MetaVal::Str(String::from("A")),
    //         ),
    //         (
    //             (
    //                 CollectMethod::First,
    //                 vec![
    //                     MetaVal::Str(String::from("A")),
    //                     MetaVal::Str(String::from("B")),
    //                     MetaVal::Str(String::from("C")),
    //                 ],
    //             ),
    //             MetaVal::Str(String::from("A")),
    //         ),
    //         (
    //             (
    //                 CollectMethod::First,
    //                 vec![],
    //             ),
    //             MetaVal::Nil,
    //         ),
    //         (
    //             (
    //                 CollectMethod::Iterate,
    //                 vec![
    //                     MetaVal::Str(String::from("A")),
    //                     MetaVal::Str(String::from("B")),
    //                     MetaVal::Str(String::from("C")),
    //                 ],
    //             ),
    //             MetaVal::Seq(
    //                 vec![
    //                     MetaVal::Str(String::from("A")),
    //                     MetaVal::Str(String::from("B")),
    //                     MetaVal::Str(String::from("C")),
    //                 ]
    //             ),
    //         ),
    //         (
    //             (
    //                 CollectMethod::Iterate,
    //                 vec![],
    //             ),
    //             MetaVal::Seq(vec![]),
    //         ),
    //     ];

    //     for (input, expected) in inputs_and_expected {
    //         let (collect_method, mvs) = input;

    //         let produced = collect_method.process(mvs);
    //         assert_eq!(expected, produced);
    //     }
    // }
}
