#![cfg(test)]

use std::fs::DirBuilder;
use std::fs::File;
use std::path::Path;

use serde_json::Value as Json;
use serde_json::Map as JsonMap;

use crate::metadata::target::Target;

pub(crate) const MEDIA_FILE_EXT: &str = "flac";

#[derive(Copy, Clone)]
pub(crate) enum Flagger<'a> {
    Predicate(fn(&[&str]) -> bool),
    FixedSet(&'a [&'a [&'a str]]),
}

impl<'a> Flagger<'a> {
    fn is_flagged(&self, name_crumbs: &[&str]) -> bool {
        match self {
            Self::Predicate(p) => p(&name_crumbs),
            Self::FixedSet(s) => s.contains(&name_crumbs),
        }
    }
}

pub(crate) const DEFAULT_FLAGGER: Flagger = Flagger::FixedSet(&[
    &["ALBUM_01", "DISC_01", "TRACK_02"],
    &["ALBUM_01", "DISC_02"],
    &["ALBUM_02"],
    &["ALBUM_02", "TRACK_01"],
    &["ALBUM_03", "DISC_01", "TRACK_01"],
    &["ALBUM_03", "DISC_01", "TRACK_02"],
    &["ALBUM_03", "DISC_01", "TRACK_03"],
    &["ALBUM_03", "DISC_02", "TRACK_01", "SUBTRACK_01"],
    &["ALBUM_03", "DISC_02", "TRACK_01", "SUBTRACK_02"],
    &["ALBUM_03", "DISC_02", "TRACK_02", "SUBTRACK_01"],
    &["ALBUM_03", "DISC_02", "TRACK_02", "SUBTRACK_02"],
    &["ALBUM_03", "DISC_02", "TRACK_03"],
    &["ALBUM_03", "DISC_02", "TRACK_04"],
]);

fn create_meta_json(name: &str, target: Target, include_flag: bool) -> Json {
    let target_str = match target {
        Target::Parent => "self",
        Target::Siblings => "item",
    };

    // let mut json = json!({
    //     "name": name,
    //     "unique_id": format!("{}_{}", name, target_str),
    //     target_str: (),
    // });

    // let mut json = json!({
    //     "const_key": "const_val",
    //     format!("{}_key", target_str): format!("{}_val", target_str),
    //     format!("{}_{}_key", name, target_str): format!("{}_{}_val", name, target_str),
    //     "overridden": format!("{}_{}", name, target_str),
    // });

    let mut json_map = JsonMap::new();
    json_map.insert("const_key".into(), "const_val".into());
    json_map.insert(format!("{}_key", target_str), format!("{}_val", target_str).into());
    json_map.insert(format!("{}_{}_key", name, target_str), format!("{}_{}_val", name, target_str).into());
    json_map.insert("overridden".into(), format!("{}_{}", name, target_str).into());

    if include_flag {
        // json.as_object_mut().map(|m| m.insert(String::from("flag"), Json::Null));

        // if let Some(map) = json.as_object_mut() {
        //     let flag_json = json!({
        //         "staggered_key": {
        //             "sub_key_a": format!("{}_sub_val_a", name),
        //             "sub_key_b": format!("{}_sub_val_b", name),
        //             "sub_key_c": {
        //                 "sub_sub_key_a": format!("{}_sub_sub_val_a", name),
        //                 "sub_sub_key_b": format!("{}_sub_sub_val_b", name),
        //             },
        //         },
        //     });
        // }

        let mut sub_sub_json_map = JsonMap::new();
        sub_sub_json_map.insert("sub_sub_key_a".into(), format!("{}_sub_sub_val_a", name).into());
        sub_sub_json_map.insert("sub_sub_key_b".into(), format!("{}_sub_sub_val_b", name).into());

        let mut sub_json_map = JsonMap::new();
        sub_json_map.insert("sub_key_a".into(), format!("{}_sub_val_a", name).into());
        sub_json_map.insert("sub_key_b".into(), format!("{}_sub_val_b", name).into());
        sub_json_map.insert("sub_key_c".into(), sub_sub_json_map.into());

        json_map.insert("staggered_key".into(), sub_json_map.into());
    }

    Json::Object(json_map)
}

pub(crate) enum Entry<'a> {
    Dir(&'a str, &'a [Entry<'a>]),
    File(&'a str),
}

impl<'a> Entry<'a> {
    /// Create this entry and its metadata recursively in a target directory.
    fn realize_and_emit(&self, root_dir: &Path, parent_name_crumbs: Vec<&str>, flagger: &Flagger) -> Json {
        let mut this_name_crumbs = parent_name_crumbs;

        // Create the actual entry (file or directory).
        let (name, is_flagged) = match self {
            Self::File(name) => {
                this_name_crumbs.push(name);

                // Just create a file, no need to recurse.
                File::create(root_dir.join(name).with_extension(MEDIA_FILE_EXT)).unwrap();

                // Flag this file if needed.
                (name, flagger.is_flagged(&this_name_crumbs))
            },
            Self::Dir(name, sub_entries) => {
                this_name_crumbs.push(name);

                let new_dir_path = root_dir.join(name);
                DirBuilder::new().create(&new_dir_path).unwrap();

                // NOTE: A little shortcut here, creating a `Library` on-the-fly
                //       and using that to recurse.
                Library { sub_entries, name, }.realize_helper(&new_dir_path, this_name_crumbs.clone(), flagger);

                // Never flag a directory at the sibling level.
                (name, false)
            }
        };

        // Create and emit the individual JSON for this entry.
        create_meta_json(name, Target::Siblings, is_flagged)
    }
}

pub(crate) struct Library<'a> {
    sub_entries: &'a [Entry<'a>],
    name: &'a str,
}

impl<'a> Library<'a> {
    pub fn realize(&self, root_dir: &Path, flagger: &Flagger) {
        self.realize_helper(root_dir, Vec::new(), flagger)
    }

    /// Create this library and its metadata recursively in a target directory.
    fn realize_helper(&self, root_dir: &Path, this_name_crumbs: Vec<&str>, flagger: &Flagger) {
        // Create a self metadata file for the root directory.
        let meta_file_path = root_dir.join("self.json");
        let meta_file = File::create(&meta_file_path).unwrap();
        let json = create_meta_json(
            self.name,
            Target::Parent,
            flagger.is_flagged(&this_name_crumbs),
        );
        serde_json::to_writer_pretty(meta_file, &json).unwrap();

        // Collect all the JSON objects created by the subentries of this library.
        let mut sub_entry_jsons = Vec::new();

        for sub_entry in self.sub_entries {
            let sub_json = sub_entry.realize_and_emit(&root_dir, this_name_crumbs.clone(), flagger);
            sub_entry_jsons.push(sub_json);
        }

        let json = Json::Array(sub_entry_jsons);

        let meta_file = File::create(root_dir.join("item.json")).unwrap();
        serde_json::to_writer_pretty(meta_file, &json).unwrap();
    }
}

pub(crate) const DEFAULT_LIBRARY: Library<'_> = Library {
    name: "ROOT",
    sub_entries: &[
        // Well-behaved album.
        Entry::Dir("ALBUM_01", &[
            Entry::Dir("DISC_01", &[
                Entry::File("TRACK_01"),
                Entry::File("TRACK_02"),
                Entry::File("TRACK_03"),
            ]),
            Entry::Dir("DISC_02", &[
                Entry::File("TRACK_01"),
                Entry::File("TRACK_02"),
                Entry::File("TRACK_03"),
            ]),
        ]),

        // Album with a disc and tracks, and loose tracks not on a disc.
        Entry::Dir("ALBUM_02", &[
            Entry::Dir("DISC_01", &[
                Entry::File("TRACK_01"),
                Entry::File("TRACK_02"),
                Entry::File("TRACK_03"),
            ]),
            Entry::File("TRACK_01"),
            Entry::File("TRACK_02"),
            Entry::File("TRACK_03"),
        ]),

        // Album with discs and tracks, and subtracks on one disc.
        Entry::Dir("ALBUM_03", &[
            Entry::Dir("DISC_01", &[
                Entry::File("TRACK_01"),
                Entry::File("TRACK_02"),
                Entry::File("TRACK_03"),
            ]),
            Entry::Dir("DISC_02", &[
                Entry::Dir("TRACK_01", &[
                    Entry::File("SUBTRACK_01"),
                    Entry::File("SUBTRACK_02"),
                ]),
                Entry::Dir("TRACK_02", &[
                    Entry::File("SUBTRACK_01"),
                    Entry::File("SUBTRACK_02"),
                ]),
                Entry::File("TRACK_03"),
                Entry::File("TRACK_04"),
            ]),
        ]),

        // Album that consists of one file.
        Entry::File("ALBUM_04"),

        // A very messed-up album.
        Entry::Dir("ALBUM_05", &[
            Entry::Dir("DISC_01", &[
                Entry::File("SUBTRACK_01"),
                Entry::File("SUBTRACK_02"),
                Entry::File("SUBTRACK_03"),
            ]),
            Entry::Dir("DISC_02", &[
                Entry::Dir("TRACK_01", &[
                    Entry::File("SUBTRACK_01"),
                    Entry::File("SUBTRACK_02"),
                ]),
            ]),
            Entry::File("TRACK_01"),
            Entry::File("TRACK_02"),
            Entry::File("TRACK_03"),
        ]),
    ]
};

#[cfg(test)]
mod tests {
    use super::*;

    use tempfile::TempDir;

    #[test]
    fn realize() {
        let temp_dir = TempDir::new().unwrap();
        let temp_dir_path = temp_dir.path();

        let flagger = Flagger::Predicate(|_| false);
        // let flagger = DEFAULT_FLAGGER;

        DEFAULT_LIBRARY.realize(&temp_dir_path, &flagger);
        // std::thread::sleep_ms(100000);
    }
}
