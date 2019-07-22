#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[derive(EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum UnaryOp {
    Collect,
    Count,
    First,
    Last,
    MinIn,
    MaxIn,
    Rev,
    Sort,
    Sum,
    Prod,
    AllEqual,
    Flatten,
    Dedup,
    Unique,
    Neg,
    Abs,
    Not,
    Noop,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[derive(EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum BinaryOp {
    Nth,
    All,
    Any,
    Find,
    Position,
    Filter,
    Map,
    StepBy,
    Chain,
    Zip,
    Skip,
    Take,
    SkipWhile,
    TakeWhile,
    And,
    Or,
    Xor,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Add,
    Sub,
    Mul,
    Div,
    Rem,
}
