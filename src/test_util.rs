#![cfg(test)]

use std::fs::{DirBuilder, File};
use std::path::Path;
use std::io::Write;
use std::thread::sleep;
use std::time::Duration;

use tempfile::Builder;
use tempfile::TempDir;

enum TEntry<'a> {
    Dir(&'a str, bool, &'a [TEntry<'a>]),
    File(&'a str, bool)
}

impl<'a> TEntry<'a> {
    pub fn name(&self) -> &str {
        match *self {
            TEntry::Dir(ref name, ..) => name,
            TEntry::File(ref name, ..) => name,
        }
    }

    pub fn include_spelunk_str(&self) -> bool {
        match *self {
            TEntry::Dir(_, b, ..) => b,
            TEntry::File(_, b, ..) => b,
        }
    }
}

const TEST_DIR_ENTRIES: &[TEntry] = &[
    // Well-behaved album.
    TEntry::Dir("ALBUM_01", false, &[
        TEntry::Dir("DISC_01", false, &[
            TEntry::File("TRACK_01", false),
            TEntry::File("TRACK_02", true),
            TEntry::File("TRACK_03", false),
        ]),
        TEntry::Dir("DISC_02", true, &[
            TEntry::File("TRACK_01", false),
            TEntry::File("TRACK_02", false),
            TEntry::File("TRACK_03", false),
        ]),
    ]),

    // Album with a disc and tracks, and loose tracks not on a disc.
    TEntry::Dir("ALBUM_02", false, &[
        TEntry::Dir("DISC_01", true, &[
            TEntry::File("TRACK_01", false),
            TEntry::File("TRACK_02", false),
            TEntry::File("TRACK_03", false),
        ]),
        TEntry::File("TRACK_01", false),
        TEntry::File("TRACK_02", true),
        TEntry::File("TRACK_03", false),
    ]),

    // Album with discs and tracks, and subtracks on one disc.
    TEntry::Dir("ALBUM_03", true, &[
        TEntry::Dir("DISC_01", true, &[
            TEntry::File("TRACK_01", true),
            TEntry::File("TRACK_02", true),
            TEntry::File("TRACK_03", true),
        ]),
        TEntry::Dir("DISC_02", true, &[
            TEntry::Dir("TRACK_01", true, &[
                TEntry::File("SUBTRACK_01", true),
                TEntry::File("SUBTRACK_02", true),
            ]),
            TEntry::Dir("TRACK_02", true, &[
                TEntry::File("SUBTRACK_01", true),
                TEntry::File("SUBTRACK_02", true),
            ]),
            TEntry::File("TRACK_03", true),
            TEntry::File("TRACK_04", true),
        ]),
    ]),

    // Album that consists of one file.
    TEntry::File("ALBUM_04", false),

    // A very messed-up album.
    TEntry::Dir("ALBUM_05", false, &[
        TEntry::Dir("DISC_01", true, &[
            TEntry::File("SUBTRACK_01", true),
            TEntry::File("SUBTRACK_02", false),
            TEntry::File("SUBTRACK_03", false),
        ]),
        TEntry::Dir("DISC_02", false, &[
            TEntry::Dir("TRACK_01", false, &[
                TEntry::File("SUBTRACK_01", true),
                TEntry::File("SUBTRACK_02", false),
            ]),
        ]),
        TEntry::File("TRACK_01", true),
        TEntry::File("TRACK_02", false),
        TEntry::File("TRACK_03", false),
    ]),
];

const MEDIA_FILE_EXT: &str = "flac";

// LEARN: Why unable to use IntoIterator<Item = Entry>?
fn create_test_dir_entries<'a, P, S>(identifier: S, target_dir_path: P, subentries: &[TEntry<'a>], db: &DirBuilder, staggered: bool)
where P: AsRef<Path>,
      S: AsRef<str>,
{
    let identifier = identifier.as_ref();
    let target_dir_path = target_dir_path.as_ref();

    // Create self meta file for this directory.
    let mut self_meta_file = File::create(target_dir_path.join("self.yml")).expect("unable to create self meta file");
    writeln!(self_meta_file, "const_key: const_val\nself_key: self_val\n{}_self_key: {}_self_val\noverridden: {}_self", identifier, identifier, identifier).expect("unable to write to self meta file");
    // writeln!(self_meta_file, "const_key: const_val").expect("unable to write to self meta file");
    // writeln!(self_meta_file, "self_key: self_val").expect("unable to write to self meta file");
    // writeln!(self_meta_file, "{}_self_key: {}_self_val", identifier, identifier).expect("unable to write to self meta file");
    // writeln!(self_meta_file, "overridden: {}_self", identifier).expect("unable to write to self meta file");

    // Create all sub-entries, and collect info to create item metadata.
    let mut item_meta_contents = String::new();
    for subentry in subentries.into_iter() {
        // helper(&subentry, &target_dir_path, db /*, imt*/);

        match *subentry {
            TEntry::File(name, ..) => {
                File::create(target_dir_path.join(name).with_extension(MEDIA_FILE_EXT)).expect("unable to create file");
            },
            TEntry::Dir(name, _, new_subentries) => {
                let new_dir_path = target_dir_path.join(name);
                db.create(&new_dir_path).expect("unable to create dir");

                create_test_dir_entries(name, new_dir_path, new_subentries, db, staggered);
            }
        }

        let entry_string = format!("- const_key: const_val\n  item_key: item_val\n  {}_item_key: {}_item_val\n  overridden: {}_item\n", subentry.name(), subentry.name(), subentry.name());
        item_meta_contents.push_str(&entry_string);

        if staggered && subentry.include_spelunk_str() {
            // Add unique meta keys that are intended for child aggregating tests.
            item_meta_contents.push_str(&format!("  staggered_key:\n"));
            item_meta_contents.push_str(&format!("    sub_key_a: {}_sub_val_a\n", subentry.name()));
            item_meta_contents.push_str(&format!("    sub_key_b: {}_sub_val_b\n", subentry.name()));
            item_meta_contents.push_str(&format!("    sub_key_c:\n"));
            item_meta_contents.push_str(&format!("      sub_sub_key_a: {}_sub_sub_val_a\n", subentry.name()));
            item_meta_contents.push_str(&format!("      sub_sub_key_b: {}_sub_sub_val_b\n", subentry.name()));
        }
    }

    // Create item meta file for all items in this directory.
    let mut item_meta_file = File::create(target_dir_path.join("item.yml")).expect("unable to create item meta file");
    item_meta_file.write_all(item_meta_contents.as_bytes()).expect("unable to write to item meta file");
}

fn create_temp_media_test_dir_helper(name: &str, staggered: bool) -> TempDir {
    let root_dir = Builder::new().suffix(name).tempdir().expect("unable to create temp directory");
    let db = DirBuilder::new();

    create_test_dir_entries("ROOT", root_dir.path(), TEST_DIR_ENTRIES, &db, staggered);

    sleep(Duration::from_millis(1));
    root_dir
}

pub fn create_temp_media_test_dir(name: &str) -> TempDir {
    create_temp_media_test_dir_helper(name, false)
}

pub fn create_temp_media_test_dir_staggered(name: &str) -> TempDir {
    create_temp_media_test_dir_helper(name, true)
}

pub(crate) struct TestUtil;

impl TestUtil {
    const FANOUT: usize = 3;
    const MAX_DEPTH: usize = 3;

    pub fn create_fanout_test_dir(name: &str) -> TempDir {
        let root_dir = Builder::new().suffix(name).tempdir().expect("unable to create temp directory");

        fn fill_dir(p: &Path, db: &DirBuilder, parent_name: &str, fanout: usize, curr_depth: usize, max_depth: usize) {
            // Create self meta file.
            let mut self_meta_file = File::create(p.join("self.json")).expect("unable to create self meta file");
            let self_lines = format!(
                r#"{{
                    "sample_string": "string",
                    "sample_integer": 27,
                    "sample_decimal": 3.1415,
                    "sample_boolean": true,
                    "sample_null": null,
                    "sample_sequence": [
                        "string",
                        27
                    ],
                    "sample_mapping": {{
                        "sample_string": "string",
                        "sample_boolean": false,
                        "sample_sequence": ["string", 27],
                        "sample_mapping": {{
                            "sample_string": "string"
                        }}
                    }},
                    "self_key": "self_val",
                    "source_meta_file": "self",
                    "target_file_name": "{}"
                }}"#,
                parent_name,
            );
            writeln!(self_meta_file, "{}", self_lines).expect("unable to write to self meta file");

            let mut item_block_entries = vec![];

            for i in 0..fanout {
                let name = if curr_depth >= max_depth {
                    // Create files.
                    let name = format!("FILE_L{}_N{}", curr_depth, i);
                    let new_path = p.join(&name);
                    File::create(&new_path).expect("unable to create item file");
                    name
                } else {
                    // Create dirs and then recurse.
                    let name = format!("DIR_L{}_N{}", curr_depth, i);
                    let new_path = p.join(&name);
                    db.create(&new_path).expect("unable to create item directory");
                    fill_dir(&new_path, &db, &name, fanout, curr_depth + 1, max_depth);
                    name
                };

                let item_block_lines = format!(
                    r#"{{
                        "sample_string": "string",
                        "sample_integer": 27,
                        "sample_decimal": 3.1415,
                        "sample_boolean": true,
                        "sample_null": null,
                        "sample_sequence": [
                            "string",
                            27
                        ],
                        "sample_mapping": {{
                            "sample_string": "string",
                            "sample_boolean": false,
                            "sample_sequence": ["string", 27],
                            "sample_mapping": {{
                                "sample_string": "string"
                            }}
                        }},
                        "item_key": "item_val",
                        "source_meta_file": "item",
                        "target_file_name": "{}"
                    }}"#,
                    name,
                );

                item_block_entries.push(item_block_lines);
            }

            // Create item meta file.
            let mut item_meta_file = File::create(p.join("item.json")).expect("unable to create item meta file");

            let item_lines = format!(
                r#"[
                    {}
                ]"#,
                item_block_entries.join(",\n"),
            );
            writeln!(item_meta_file, "{}", item_lines).expect("unable to write to item meta file");
        }

        let db = DirBuilder::new();

        fill_dir(root_dir.path(), &db, "ROOT", Self::FANOUT, 0, Self::MAX_DEPTH);

        std::thread::sleep(Duration::from_millis(1));
        root_dir
    }
}

#[cfg(test)]
mod tests {
    use super::TestUtil;

    #[test]
    fn test_create_fanout_test_dir() {
        let temp_dir = TestUtil::create_fanout_test_dir("test_create_fanout_test_dir");
    }
}
