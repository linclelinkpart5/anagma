
use crate::metadata::types::MetaVal;
use crate::updated_scripting::Error;
use crate::updated_scripting::ops::predicate::Predicate;

pub enum Converter {
    Collect,
    Sort,
    Rev,
    Count,
    First,
    Last,
    MinIn,
    MaxIn,
    Sum,
    Prod,
    Flatten,
    Dedup,
    Unique,
    Neg,
    Abs,
    Pass,
    Keys,
    Vals,
    Pick,
    Load,
    Predicate(Predicate),
}

impl Converter {
    pub fn convert(&self, mv: MetaVal) -> Result<MetaVal, Error> {
        match self {
            &Self::Predicate(ref pred) => pred.test(&mv).map(MetaVal::from),
            _ => Ok(MetaVal::Nil),
        }
    }
}
