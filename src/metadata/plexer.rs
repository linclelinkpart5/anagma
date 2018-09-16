use std::path::Path;
use std::path::PathBuf;
use std::collections::HashMap;

use failure::Error;

use metadata::structure::MetaStructure;
use metadata::types::MetaBlock;

pub struct MetaPlexer;

pub type MetaPlexResult = HashMap<PathBuf, MetaBlock>;

impl MetaPlexer {
    pub fn plex<II, P>(meta_structure: MetaStructure, item_paths: II) -> Result<MetaPlexResult, Error>
    where II: IntoIterator<Item = P>,
          P: AsRef<Path>,
    {
        let mut item_paths = item_paths.into_iter();

        let mut result = MetaPlexResult::new();

        match meta_structure {
            MetaStructure::One(mb) => {
                // Exactly one item path is expected.
                if let Some(item_path) = item_paths.next() {
                    // Raise error if there are still more paths to process.
                    if let Some(_) = item_paths.next() {
                        let extra_count = item_paths.count();

                        bail!(format!("expected exactly 1 item path, found {}", 1 + extra_count));
                    }

                    result.insert(item_path.as_ref().to_path_buf(), mb);
                }
                else {
                    bail!(format!("expected exactly 1 item path, found {}", 0));
                }
            },
            MetaStructure::Seq(mbs) => {
                let collected_item_paths: Vec<_> = item_paths.collect();

                let expected_num = mbs.len();
                let produced_num = collected_item_paths.len();

                match expected_num == produced_num {
                    false => {
                        // TODO: Warn, but do not error.
                        bail!(format!("expected exactly {} item path{}, found {}",
                            expected_num,
                            if expected_num == 1 { "" } else { "s" },
                            produced_num,
                        ));
                    },
                    true => {
                        for (item_path, mb) in collected_item_paths.iter().zip(mbs) {
                            result.insert(item_path.as_ref().to_path_buf(), mb);
                        }
                    },
                };
            },
            MetaStructure::Map(mbm) => {
                // Create a pool of found tags.
                let mut found = hashset![];

                found.insert(0);

                for item_path in item_paths {
                    // Use the file name of the item path as a key into the mapping.
                    let raw_key = match item_path.as_ref().file_name() {
                        Some(file_name) => file_name,
                        None => { bail!("item path does not have a file name"); },
                    };

                    let key = match raw_key.to_str() {
                        Some(s) => s,
                        None => { bail!("item path file name is not valid"); },
                    };

                    match mbm.get(key) {
                        Some(mb) => {},
                        None => {},
                    };
                }
            },
        };

        Ok(result)
    }
}
