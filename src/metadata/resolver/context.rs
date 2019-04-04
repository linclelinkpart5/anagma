use std::path::Path;

use config::selection::Selection;
use config::sort_order::SortOrder;
use config::meta_format::MetaFormat;
use metadata::types::MetaKey;

pub struct ResolverContext<'rc> {
    pub current_key_path: Vec<&'rc MetaKey>,
    pub current_item_file_path: &'rc Path,
    pub meta_format: MetaFormat,
    pub selection: &'rc Selection,
    pub sort_order: SortOrder,
}
