use crate::metadata::types::MetaVal;
use crate::functions::Error;

#[derive(Clone, Copy, Debug)]
pub enum IterConsumer {
    Nth,
    All,
    Any,
    Find,
    Position,
}

impl IterConsumer {
    pub fn process<'mv>(&self, _it: impl Iterator<Item = Result<MetaVal<'mv>, Error>>) -> Result<MetaVal<'mv>, Error> {
        match self {
            _ => Ok(MetaVal::Nil),
        }
    }
}
