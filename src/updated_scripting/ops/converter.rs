
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
