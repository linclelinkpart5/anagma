//! Data representations of meta files.

use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

use crate::types::Value;

/// A metadata block, consisting of key-value pairs (aka "fields").
pub type Block = BTreeMap<String, Value>;

/// Represents a collection of metadata blocks.
/// Metadata blocks may be untagged, or tagged with a file name.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Blocks {
    Untagged(Vec<Block>),
    Tagged(HashMap<String, Block>),
}

impl Default for Blocks {
    fn default() -> Self {
        Self::Untagged(Default::default())
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Metadata {
    #[serde(default)]
    album: Block,
    #[serde(default)]
    tracks: Blocks,
}

#[cfg(test)]
mod tests {
    use super::*;

    use indoc::indoc;

    #[test]
    fn deserialize() {
        let text = indoc! {"
            [album]
            artist = 'Relleo'
            title = 'RelleoX'

            [[tracks]]
            rating = 5
            artist = ['Relleo', 'Invinceable']
            title = 'I KNOW'

            [[tracks]]
            rating = 5
            title = 'NO LOVE'
        "};

        println!("{}", text);

        let metadata: Metadata = toml::from_str(&text).unwrap();

        println!("{:?}", metadata);

        for entry in glob::glob("test/inputs/metadata/toml/*.toml").unwrap() {
            let path = entry.unwrap();

            let text = std::fs::read_to_string(&path).unwrap();

            println!("Parsing TOML file: {}", path.display());
            println!("------------- TEXT -------------");
            println!("{}", text);
            println!("--------------------------------");

            let metadata: Metadata = toml::from_str(&text).unwrap();

            println!("----------- METADATA -----------");
            println!("{:#?}", metadata);
            println!("--------------------------------");
        }
    }
}
