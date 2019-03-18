use std::collections::BTreeMap;

use bigdecimal::BigDecimal;

use metadata::types::MetaKey;
use metadata::types::MetaVal;

#[derive(Debug)]
pub enum Error {
    EmptyStack,
    UnexpectedType{expected: ParamType, found: ParamType},
    ZeroInteger,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::EmptyStack => write!(f, "empty stack"),
            Self::UnexpectedType{expected, found} => write!(f, "expected {}, found {}",  expected, found),
            Self::ZeroInteger => write!(f, "zero integer"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Self::EmptyStack => None,
            Self::UnexpectedType{..} => None,
            Self::ZeroInteger => None,
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
enum StackItem {
    Val(MetaVal),
    UnaryOp(UnaryOp),
    BinaryOp(BinaryOp),
    NNInteger(usize),
    PosInteger(usize),
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

impl From<usize> for StackItem {
    fn from(ui: usize) -> Self {
        Self::NNInteger(ui)
    }
}

impl StackItem {
    fn validate(&self) -> Result<(), Error> {
        match self {
            &Self::PosInteger(i) => if i > 0 { Ok(()) } else { Err(Error::ZeroInteger) },
            _ => Ok(()),
        }
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
    NNInteger,
    PosInteger,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Number {
    Integer(i64),
    Decimal(BigDecimal),
}

impl PartialOrd for Number {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Number {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Self::Integer(l), Self::Integer(r)) => l.cmp(r),
            (Self::Integer(l), Self::Decimal(r)) => BigDecimal::from(*l).cmp(r),
            (Self::Decimal(l), Self::Integer(r)) => l.cmp(&BigDecimal::from(*r)),
            (Self::Decimal(l), Self::Decimal(r)) => l.cmp(r),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum SItem {
    Null,
    Text(String),
    Sequence(Vec<SItem>),
    Mapping(BTreeMap<MetaKey, SItem>),
    Boolean(bool),
    Number(Number),
    UnaryOp(UnaryOp),
    BinaryOp(BinaryOp),
}

impl From<MetaVal> for SItem {
    fn from(meta_val: MetaVal) -> Self {
        match meta_val {
            MetaVal::Nil => SItem::Null,
            MetaVal::Str(s) => SItem::Text(s),
            MetaVal::Seq(s) => SItem::Sequence(s.into_iter().map(|v| v.into()).collect()),
            MetaVal::Map(m) => SItem::Mapping(m.into_iter().map(|(k, v)| (k, v.into())).collect()),
            MetaVal::Bul(b) => SItem::Boolean(b),
            MetaVal::Int(i) => SItem::Number(Number::Integer(i)),
            MetaVal::Dec(d) => SItem::Number(Number::Decimal(d)),
        }
    }
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
            Self::NNInteger => write!(f, "non-negative integer"),
            Self::PosInteger => write!(f, "positive integer"),
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
            &StackItem::NNInteger(..) => Self::NNInteger,
            &StackItem::PosInteger(..) => Self::PosInteger,
        }
    }
}

impl ParamType {
    fn process_stack(&self, stack: &mut Vec<StackItem>) -> Result<StackItem, Error> {
        match stack.pop() {
            None => Err(Error::EmptyStack),
            Some(stack_item) => {
                let stack_item_type: ParamType = (&stack_item).into();

                if self == &stack_item_type {
                    stack_item.validate()?;

                    Ok(stack_item)
                }
                else {
                    Err(Error::UnexpectedType{expected: *self, found: stack_item_type})
                }
            },
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
            Self::Nth => (ParamType::Sequence, ParamType::NNInteger),
            Self::StepBy => (ParamType::Sequence, ParamType::PosInteger),
            Self::Chain => (ParamType::Sequence, ParamType::Sequence),
            Self::Zip => (ParamType::Sequence, ParamType::Sequence),
            Self::Map => (ParamType::Sequence, ParamType::UnaryOp),
            Self::Filter => (ParamType::Sequence, ParamType::UnaryOp),
            Self::SkipWhile => (ParamType::Sequence, ParamType::UnaryOp),
            Self::TakeWhile => (ParamType::Sequence, ParamType::UnaryOp),
            Self::Skip => (ParamType::Sequence, ParamType::PosInteger),
            Self::Take => (ParamType::Sequence, ParamType::PosInteger),
            Self::Fold => (ParamType::Sequence, ParamType::BinaryOp),
            Self::All => (ParamType::Sequence, ParamType::UnaryOp),
            Self::Any => (ParamType::Sequence, ParamType::UnaryOp),
            Self::Find => (ParamType::Sequence, ParamType::UnaryOp),
            Self::Position => (ParamType::Sequence, ParamType::UnaryOp),
            Self::Interleave => (ParamType::Sequence, ParamType::Any),
            Self::Intersperse => (ParamType::Sequence, ParamType::Sequence),
            Self::Chunks => (ParamType::Sequence, ParamType::PosInteger),
            Self::Windows => (ParamType::Sequence, ParamType::PosInteger),
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
            Self::Pad => (ParamType::Sequence, ParamType::PosInteger, ParamType::Any),
        }
    }
}
