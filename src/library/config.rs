//! Provides configuration options for a Taggu library, both programmatically and via YAML files.

use library::selection::Selection;
use library::sort_order::SortOrder;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    selection: Selection,
    sort_order: SortOrder,
}

#[cfg(test)]
mod tests {
    use serde_yaml;

    use super::Config;
    use super::Selection;
    use super::SortOrder;

    #[test]
    fn test_serialization() {
        let config = Config {
            selection: Selection::IsFile,
            sort_order: SortOrder::Name,
        };

        let s = serde_yaml::to_string(&config).unwrap();

        println!("{}", s);
    }
}
