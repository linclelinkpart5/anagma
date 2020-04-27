#![cfg(test)]

use std::collections::HashSet;
use std::fs::DirBuilder;
use std::fs::File;
use std::path::Path;

use serde_json::Value as Json;

use serde_json::json;

use crate::metadata::target::Target;

pub(crate) const MEDIA_FILE_EXT: &str = "flac";

pub(crate) enum Flagger<'a> {
    Iterator(Box<dyn Iterator<Item = bool>>),
    Predicate(Box<dyn Fn(&str, Target) -> bool>),
    FixedSet(&'a HashSet<(&'a str, Target)>),
}

impl<'a> Flagger<'a> {
    fn is_flagged(&mut self, name: &str, target: Target) -> bool {
        match self {
            Self::Iterator(i) => i.next().unwrap_or(false),
            Self::Predicate(p) => p(&name, target),
            Self::FixedSet(s) => s.contains(&(name, target)),
        }
    }
}

pub(crate) enum NEntry<'a> {
    Dir(&'a str, &'a [NEntry<'a>]),
    File(&'a str),
}

impl<'a> NEntry<'a> {
    /// Create this entry and its metadata recursively in a target directory.
    pub fn realize_and_emit(&self, root_dir: &Path, flagger: &mut Flagger) -> Json {
        // Create the actual entry (file or directory).
        let name = match self {
            Self::File(name) => {
                // Just create a file, no need to recurse.
                File::create(root_dir.join(name).with_extension(MEDIA_FILE_EXT)).unwrap();
                name
            },
            Self::Dir(name, sub_entries) => {
                let new_dir_path = root_dir.join(name);
                DirBuilder::new().create(&new_dir_path).unwrap();

                // NOTE: A little shortcut here, creating a `NLibrary` on-the-fly
                //       and using that to recurse.
                NLibrary { sub_entries, name, }.realize(&new_dir_path, flagger);

                name
            }
        };

        // Create and emit the individual JSON for this entry.
        create_meta_json(
            name,
            Target::Siblings,
            flagger.is_flagged(&name, Target::Siblings),
        )
    }
}

pub(crate) enum Entry<'a> {
    Dir(&'a str, &'a [Entry<'a>], Option<Target>),
    File(&'a str, Option<Target>),
}

impl<'a> Entry<'a> {
    /// Create this entry and its metadata recursively in a target directory.
    pub fn realize_and_emit(&self, root_dir: &Path) -> Json {
        // Create the actual entry (file or directory).
        let (name, opt_flag) = match self {
            Self::File(name, opt_flag) => {
                // Just create a file, no need to recurse.
                File::create(root_dir.join(name).with_extension(MEDIA_FILE_EXT)).unwrap();
                (name, opt_flag)
            },
            Self::Dir(name, sub_entries, opt_flag) => {
                let new_dir_path = root_dir.join(name);
                DirBuilder::new().create(&new_dir_path).unwrap();

                // NOTE: A little shortcut here, creating a library on-the-fly
                //       and using that to recurse.
                Library { sub_entries, name, }.realize(&new_dir_path);

                (name, opt_flag)
            }
        };

        // Create and emit the individual JSON for this entry.
        create_meta_json(
            name,
            Target::Siblings,
            opt_flag.map(|t| t == Target::Siblings).unwrap_or(false),
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

pub(crate) struct NLibrary<'a> {
    sub_entries: &'a [NEntry<'a>],
    name: &'a str,
}

impl<'a> NLibrary<'a> {
    /// Create this library and its metadata recursively in a target directory.
    pub fn realize(&self, root_dir: &Path, flagger: &mut Flagger) {
        // Create a self metadata file for the root directory.
        let meta_file_path = root_dir.join("self.json");
        let meta_file = File::create(&meta_file_path).unwrap();
        let json = create_meta_json(
            self.name,
            Target::Parent,
            flagger.is_flagged(self.name, Target::Parent),
        );
        serde_json::to_writer_pretty(meta_file, &json).unwrap();

        // Collect all the JSON objects created by the subentries of this library.
        let mut sub_entry_jsons = Vec::new();

        for sub_entry in self.sub_entries {
            let sub_json = sub_entry.realize_and_emit(&root_dir, flagger);
            sub_entry_jsons.push(sub_json);
        }

        let json = Json::Array(sub_entry_jsons);

        let mut meta_file = File::create(root_dir.join("item.json")).unwrap();
        serde_json::to_writer_pretty(meta_file, &json).unwrap();
    }
}

pub(crate) struct Library<'a> {
    sub_entries: &'a [Entry<'a>],
    name: &'a str,
}

impl<'a> Library<'a> {
    /// Create this library and its metadata recursively in a target directory.
    pub fn realize(&self, root_dir: &Path) {
        // Create a self metadata file for the root directory.
        let meta_file_path = root_dir.join("self.json");
        let meta_file = File::create(&meta_file_path).unwrap();

        let json = create_meta_json(self.name, Target::Parent, false);
        serde_json::to_writer_pretty(meta_file, &json).unwrap();

        // Collect all the JSON objects created by the subentries of this library.
        let mut sub_entry_jsons = Vec::new();

        for sub_entry in self.sub_entries {
            let sub_json = sub_entry.realize_and_emit(&root_dir);
            sub_entry_jsons.push(sub_json);
        }

        let json = Json::Array(sub_entry_jsons);

        let mut meta_file = File::create(root_dir.join("item.json")).unwrap();
        serde_json::to_writer_pretty(meta_file, &json).unwrap();
    }
}

const MAIN_LIBRARY: NLibrary<'_> = NLibrary {
    name: "ROOT",
    sub_entries: &[
        // Well-behaved album.
        NEntry::Dir("ALBUM_01", &[
            NEntry::Dir("DISC_01", &[
                NEntry::File("TRACK_01"),
                NEntry::File("TRACK_02"),
                NEntry::File("TRACK_03"),
            ]),
            NEntry::Dir("DISC_02", &[
                NEntry::File("TRACK_01"),
                NEntry::File("TRACK_02"),
                NEntry::File("TRACK_03"),
            ]),
        ]),

        // Album with a disc and tracks, and loose tracks not on a disc.
        NEntry::Dir("ALBUM_02", &[
            NEntry::Dir("DISC_01", &[
                NEntry::File("TRACK_01"),
                NEntry::File("TRACK_02"),
                NEntry::File("TRACK_03"),
            ]),
            NEntry::File("TRACK_01"),
            NEntry::File("TRACK_02"),
            NEntry::File("TRACK_03"),
        ]),

        // Album with discs and tracks, and subtracks on one disc.
        NEntry::Dir("ALBUM_03", &[
            NEntry::Dir("DISC_01", &[
                NEntry::File("TRACK_01"),
                NEntry::File("TRACK_02"),
                NEntry::File("TRACK_03"),
            ]),
            NEntry::Dir("DISC_02", &[
                NEntry::Dir("TRACK_01", &[
                    NEntry::File("SUBTRACK_01"),
                    NEntry::File("SUBTRACK_02"),
                ]),
                NEntry::Dir("TRACK_02", &[
                    NEntry::File("SUBTRACK_01"),
                    NEntry::File("SUBTRACK_02"),
                ]),
                NEntry::File("TRACK_03"),
                NEntry::File("TRACK_04"),
            ]),
        ]),

        // Album that consists of one file.
        NEntry::File("ALBUM_04"),

        // A very messed-up album.
        NEntry::Dir("ALBUM_05", &[
            NEntry::Dir("DISC_01", &[
                NEntry::File("SUBTRACK_01"),
                NEntry::File("SUBTRACK_02"),
                NEntry::File("SUBTRACK_03"),
            ]),
            NEntry::Dir("DISC_02", &[
                NEntry::Dir("TRACK_01", &[
                    NEntry::File("SUBTRACK_01"),
                    NEntry::File("SUBTRACK_02"),
                ]),
            ]),
            NEntry::File("TRACK_01"),
            NEntry::File("TRACK_02"),
            NEntry::File("TRACK_03"),
        ]),
    ]
};
