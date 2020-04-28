#![cfg(test)]

use std::fs::DirBuilder;
use std::fs::File;
use std::path::Path;

use serde_json::Value as Json;

use serde_json::json;

use crate::metadata::target::Target;

pub(crate) const MEDIA_FILE_EXT: &str = "flac";

#[derive(Copy, Clone)]
pub(crate) enum Flagger<'a> {
    Predicate(fn(&[&str], Target) -> bool),
    FixedSet(&'a [(&'a [&'a str], Target)]),
}

impl<'a> Flagger<'a> {
    fn is_flagged(&self, name_crumbs: &[&str], target: Target) -> bool {
        println!("Testing: {:?}, {:?}", name_crumbs, target);
        match self {
            Self::Predicate(p) => p(&name_crumbs, target),
            Self::FixedSet(s) => s.contains(&(name_crumbs, target)),
        }
    }
}

const DEFAULT_FLAGGER: Flagger = Flagger::FixedSet(&[
    (&["ALBUM_01", "DISC_01", "TRACK_02"], Target::Parent),
    (&["ALBUM_01", "DISC_02"], Target::Parent),
    (&["ALBUM_02"], Target::Parent),
    (&["ALBUM_02", "TRACK_01"], Target::Parent),
    (&["ALBUM_03", "DISC_01", "TRACK_01"], Target::Parent),
    (&["ALBUM_03", "DISC_01", "TRACK_02"], Target::Parent),
    (&["ALBUM_03", "DISC_01", "TRACK_03"], Target::Parent),
    (&["ALBUM_03", "DISC_02", "TRACK_01", "SUBTRACK_01"], Target::Parent),
    (&["ALBUM_03", "DISC_02", "TRACK_01", "SUBTRACK_02"], Target::Parent),
    (&["ALBUM_03", "DISC_02", "TRACK_02", "SUBTRACK_01"], Target::Parent),
    (&["ALBUM_03", "DISC_02", "TRACK_02", "SUBTRACK_02"], Target::Parent),
    (&["ALBUM_03", "DISC_02", "TRACK_03"], Target::Parent),
    (&["ALBUM_03", "DISC_02", "TRACK_04"], Target::Parent),
]);

pub(crate) enum Entry<'a> {
    Dir(&'a str, &'a [Entry<'a>]),
    File(&'a str),
}

impl<'a> Entry<'a> {
    /// Create this entry and its metadata recursively in a target directory.
    pub fn realize_and_emit(&self, root_dir: &Path, name_crumbs: Vec<&str>, flagger: &Flagger) -> Json {
        let mut new_name_crumbs = name_crumbs;

        // Create the actual entry (file or directory).
        let name = match self {
            Self::File(name) => {
                new_name_crumbs.push(name);

                // Just create a file, no need to recurse.
                File::create(root_dir.join(name).with_extension(MEDIA_FILE_EXT)).unwrap();
                name
            },
            Self::Dir(name, sub_entries) => {
                new_name_crumbs.push(name);

                let new_dir_path = root_dir.join(name);
                DirBuilder::new().create(&new_dir_path).unwrap();

                // NOTE: A little shortcut here, creating a `Library` on-the-fly
                //       and using that to recurse.
                Library { sub_entries, name, }.realize_helper(&new_dir_path, new_name_crumbs.clone(), flagger);

                name
            }
        };

        // Create and emit the individual JSON for this entry.
        println!("{:?}", new_name_crumbs);
        create_meta_json(
            name,
            Target::Siblings,
            flagger.is_flagged(&new_name_crumbs, Target::Siblings),
        )
    }
}

fn create_meta_json(name: &str, target: Target, include_flag: bool) -> Json {
    let target_str = match target {
        Target::Parent => "parent",
        Target::Siblings => "siblings",
    };

    let mut json = json!({
        "name": name,
        "unique_id": format!("{}_{}", name, target_str),
        target_str: (),
    });

    if include_flag {
        json.as_object_mut().map(|m| m.insert(String::from("flag"), Json::Null));
    }

    json
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
    fn realize_helper(&self, root_dir: &Path, name_crumbs: Vec<&str>, flagger: &Flagger) {
        // Create a self metadata file for the root directory.
        let meta_file_path = root_dir.join("self.json");
        let meta_file = File::create(&meta_file_path).unwrap();
        let json = create_meta_json(
            self.name,
            Target::Parent,
            flagger.is_flagged(&name_crumbs, Target::Parent),
        );
        serde_json::to_writer_pretty(meta_file, &json).unwrap();

        // Collect all the JSON objects created by the subentries of this library.
        let mut sub_entry_jsons = Vec::new();

        for sub_entry in self.sub_entries {
            let sub_json = sub_entry.realize_and_emit(&root_dir, name_crumbs.clone(), flagger);
            sub_entry_jsons.push(sub_json);
        }

        let json = Json::Array(sub_entry_jsons);

        let meta_file = File::create(root_dir.join("item.json")).unwrap();
        serde_json::to_writer_pretty(meta_file, &json).unwrap();
    }
}

const MAIN_LIBRARY: Library<'_> = Library {
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

        // let flagger = Flagger::Predicate(|_, _| false);
        let flagger = DEFAULT_FLAGGER;

        MAIN_LIBRARY.realize(&temp_dir_path, &flagger);
        println!("{}", temp_dir_path.display());
        std::thread::sleep_ms(100000);
    }
}
