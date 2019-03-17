use metadata::types::MetaVal;

#[derive(Debug)]
pub enum Error {
    EmptyStack,
    UnexpectedType{expected: ParamType, found: ParamType},
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::EmptyStack => write!(f, "empty stack"),
            Self::UnexpectedType{expected, found} => write!(f, "expected {}, found {}",  expected, found),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Self::EmptyStack => None,
            Self::UnexpectedType{..} => None,
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum StackItem {
    Val(MetaVal),
    UnaryOp(UnaryOp),
    BinaryOp(BinaryOp),
}

impl From<MetaVal> for StackItem {
    fn from(meta_val: MetaVal) -> Self {
        Self::Val(meta_val)
    }
}

impl From<UnaryOp> for StackItem {
    fn from(unary_op: UnaryOp) -> Self {
        Self::UnaryOp(unary_op)
    }
}

impl From<BinaryOp> for StackItem {
    fn from(binary_op: BinaryOp) -> Self {
        Self::BinaryOp(binary_op)
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum ParamType {
    Any,
    Text,
    Sequence,
    Mapping,
    Boolean,
    Number,
    Integer,
    Float,
    Null,
    UnaryOp,
    BinaryOp,
}

impl std::fmt::Display for ParamType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::Any => write!(f, "any"),
            Self::Text => write!(f, "text"),
            Self::Sequence => write!(f, "sequence"),
            Self::Mapping => write!(f, "mapping"),
            Self::Boolean => write!(f, "boolean"),
            Self::Number => write!(f, "number"),
            Self::Integer => write!(f, "integer"),
            Self::Float => write!(f, "float"),
            Self::Null => write!(f, "null"),
            Self::UnaryOp => write!(f, "unary op"),
            Self::BinaryOp => write!(f, "binary op"),
        }
    }
}

impl From<&MetaVal> for ParamType {
    fn from(meta_val: &MetaVal) -> Self {
        match meta_val {
            &MetaVal::Nil => Self::Null,
            &MetaVal::Str(..) => Self::Text,
            &MetaVal::Seq(..) => Self::Sequence,
            &MetaVal::Map(..) => Self::Mapping,
            &MetaVal::Int(..) => Self::Integer,
            &MetaVal::Bul(..) => Self::Boolean,
            &MetaVal::Dec(..) => Self::Float,
        }
    }
}

impl From<&StackItem> for ParamType {
    fn from(stack_item: &StackItem) -> Self {
        match stack_item {
            &StackItem::Val(ref meta_val) => meta_val.into(),
            &StackItem::UnaryOp(..) => Self::UnaryOp,
            &StackItem::BinaryOp(..) => Self::BinaryOp,
        }
    }
}

// pub trait Op {
//     const ARITY: usize;

//     fn input_types(&self) -> &'static [ParamType; Self::ARITY];
//     fn output_type(&self) -> ParamType;

//     fn process_stack(&self, stack: &mut Vec<StackItem>) -> Result<StackItem, Error> {
//         Ok(StackItem::Val(MetaVal::Nil))
//     }
// }

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
