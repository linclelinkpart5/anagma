
use std::collections::BTreeMap;

use crate::util::Number;
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

    NthA(usize),
    NthB(Vec<MetaVal>),
    All(Predicate),
    Any(Predicate),
    Find(Predicate),
    Pos(Predicate),
    Filter(Predicate),
    SkipWhile(Predicate),
    TakeWhile(Predicate),
    Map(Box<Converter>),
    SkipA(usize),
    SkipB(Vec<MetaVal>),
    TakeA(usize),
    TakeB(Vec<MetaVal>),
    StepByA(usize),
    StepByB(Vec<MetaVal>),
    ChainA(Vec<MetaVal>),
    ChainB(Vec<MetaVal>),
    ZipA(Vec<MetaVal>),
    ZipB(Vec<MetaVal>),
    IntersperseA(MetaVal),
    IntersperseB(Vec<MetaVal>),
    RoundRobinA(Vec<MetaVal>),
    RoundRobinB(Vec<MetaVal>),
    Add(Number),
    SubA(Number),
    SubB(Number),
    Mul(Number),
    DivA(Number),
    DivB(Number),
    RemA(Number),
    RemB(Number),
    LookupA(String),
    LookupB(BTreeMap<String, MetaVal>),
    SaveA(MetaVal),
    SaveB(String),
    Min(Number),
    Max(Number),
}

impl Converter {
    pub fn convert(&self, mv: MetaVal) -> Result<MetaVal, Error> {
        match self {
            &Self::Predicate(ref pred) => pred.test(&mv).map(MetaVal::from),
            _ => Ok(MetaVal::Nil),
        }
    }
}
