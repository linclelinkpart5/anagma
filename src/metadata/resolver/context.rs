use std::path::Path;

use config::selection::Selection;
use config::sort_order::SortOrder;
use config::meta_format::MetaFormat;
use metadata::types::MetaKey;

pub struct ResolverContext<'k, 'p, 's> {
    pub current_key_path: Vec<&'k MetaKey>,
    pub current_item_file_path: &'p Path,
    pub meta_format: MetaFormat,
    pub selection: &'s Selection,
    pub sort_order: SortOrder,
}
