#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum ParamType {
    Text,
    Sequence,
    Mapping,
    Boolean,
    Number,
    Integer,
    Float,
    Any,
    Null,
    UnaryOp,
    BinaryOp,
}

pub enum Function {
    Unary(UnaryOp),
    Binary(BinaryOp),
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum UnaryOp {
    Count,
    First,
    Last,
    Enum,
    Flatten,
    FlattenRec,
    Max,
    Min,
    Rev,
    Sum,
    Product,
    Dedup,
    Unique,
    AllEqual,
    Sort,
}

impl UnaryOp {
    pub fn input_type_spec(&self) -> ParamType {
        match *self {
            Self::Count => ParamType::Sequence,
            Self::First => ParamType::Sequence,
            Self::Last => ParamType::Sequence,
            Self::Enum => ParamType::Sequence,
            Self::Flatten => ParamType::Sequence,
            Self::FlattenRec => ParamType::Sequence,
            Self::Max => ParamType::Sequence,
            Self::Min => ParamType::Sequence,
            Self::Rev => ParamType::Sequence,
            Self::Sum => ParamType::Sequence,
            Self::Product => ParamType::Sequence,
            Self::Dedup => ParamType::Sequence,
            Self::Unique => ParamType::Sequence,
            Self::AllEqual => ParamType::Sequence,
            Self::Sort => ParamType::Sequence,
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum BinaryOp {
    Eq,
    Ne,
    Gt,
    Ge,
    Lt,
    Le,
    Nth,
    StepBy,
    Chain,
    Zip,
    Map,
    Filter,
    SkipWhile,
    TakeWhile,
    Skip,
    Take,
    Fold,
    All,
    Any,
    Find,
    Position,
    Interleave,
    Intersperse,
    Chunks,
    Windows,
    Merge,
}

impl BinaryOp {
    pub fn input_type_spec(&self) -> (ParamType, ParamType) {
        match *self {
            Self::Eq => (ParamType::Any, ParamType::Any),
            Self::Ne => (ParamType::Any, ParamType::Any),
            Self::Gt => (ParamType::Any, ParamType::Any),
            Self::Ge => (ParamType::Any, ParamType::Any),
            Self::Lt => (ParamType::Any, ParamType::Any),
            Self::Le => (ParamType::Any, ParamType::Any),
            Self::Nth => (ParamType::Sequence, ParamType::Integer),
            Self::StepBy => (ParamType::Sequence, ParamType::Integer),
            Self::Chain => (ParamType::Sequence, ParamType::Sequence),
            Self::Zip => (ParamType::Sequence, ParamType::Sequence),
            Self::Map => (ParamType::Sequence, ParamType::UnaryOp),
            Self::Filter => (ParamType::Sequence, ParamType::UnaryOp),
            Self::SkipWhile => (ParamType::Sequence, ParamType::UnaryOp),
            Self::TakeWhile => (ParamType::Sequence, ParamType::UnaryOp),
            Self::Skip => (ParamType::Sequence, ParamType::Integer),
            Self::Take => (ParamType::Sequence, ParamType::Integer),
            Self::Fold => (ParamType::Sequence, ParamType::BinaryOp),
            Self::All => (ParamType::Sequence, ParamType::UnaryOp),
            Self::Any => (ParamType::Sequence, ParamType::UnaryOp),
            Self::Find => (ParamType::Sequence, ParamType::UnaryOp),
            Self::Position => (ParamType::Sequence, ParamType::UnaryOp),
            Self::Interleave => (ParamType::Sequence, ParamType::Any),
            Self::Intersperse => (ParamType::Sequence, ParamType::Sequence),
            Self::Chunks => (ParamType::Sequence, ParamType::Integer),
            Self::Windows => (ParamType::Sequence, ParamType::Integer),
            Self::Merge => (ParamType::Sequence, ParamType::Sequence),
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum TernaryOp {
    Pad,
}

impl TernaryOp {
    pub fn input_type_spec(&self) -> (ParamType, ParamType, ParamType) {
        match *self {
            Self::Pad => (ParamType::Sequence, ParamType::Integer, ParamType::Any),
        }
    }
}
