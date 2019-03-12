use metadata::types::MetaKey;
use metadata::types::MetaVal;

#[derive(Copy, Clone, Debug)]
pub enum FallbackIterKind {
    Parents,
    ChildrenDepth,
    ChildrenBreadth,
}

pub struct FallbackIter<'k> {
    target_key_path: Vec<&'k MetaKey>,
    kind: FallbackIterKind,
}

impl<'k> Iterator for FallbackIter<'k> {
    type Item = MetaVal;

    fn next(&mut self) -> Option<Self::Item> {
        Some(MetaVal::Nil)
    }
}
