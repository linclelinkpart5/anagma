use std::path::Path;

use crate::config::selection::Selection;
use crate::config::sort_order::SortOrder;
use crate::config::meta_format::MetaFormat;
use crate::metadata::types::MetaKeyPath;

pub struct ResolverContext<'rc> {
    pub current_key_path: MetaKeyPath<'rc>,
    pub current_item_file_path: &'rc Path,
    pub meta_format: MetaFormat,
    pub selection: &'rc Selection,
    pub sort_order: SortOrder,
}
