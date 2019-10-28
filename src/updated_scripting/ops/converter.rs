
use crate::metadata::types::MetaVal;
use crate::updated_scripting::ops::predicate::Predicate1;

#[derive(Clone, Debug)]
pub enum Converter1 {
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
    Predicate(Predicate1),
    Partial(MetaVal, Converter2),
}

#[derive(Clone, Debug)]
pub enum Converter2 {
}
