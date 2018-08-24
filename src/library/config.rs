//! Provides configuration options for a Taggu library, both programmatically and via YAML files.

use library::selection::Selection;
use library::sort_order::SortOrder;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    selection: Selection,
    sort_order: SortOrder,
}
