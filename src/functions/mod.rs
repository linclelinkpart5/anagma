use std::collections::BTreeMap;

use bigdecimal::BigDecimal;

use metadata::types::MetaKey;
use metadata::types::MetaVal;

#[derive(Debug)]
pub enum Error {
    EmptyStack,
    UnexpectedType{expected: &'static str, found: &'static str},
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

impl From<i64> for Number {
    fn from(n: i64) -> Self {
        Self::Integer(n)
    }
}

impl From<BigDecimal> for Number {
    fn from(n: BigDecimal) -> Self {
        Self::Decimal(n)
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum StackOperand {
    Null,
    Text(String),
    Sequence(Vec<Self>),
    Mapping(BTreeMap<MetaKey, Self>),
    Boolean(bool),
    Number(Number),
    UnaryOp(UnaryOp),
    BinaryOp(BinaryOp),
}

impl From<MetaVal> for StackOperand {
    fn from(meta_val: MetaVal) -> Self {
        match meta_val {
            MetaVal::Nil => Self::Null,
            MetaVal::Str(s) => Self::Text(s),
            MetaVal::Seq(s) => Self::Sequence(s.into_iter().map(|v| v.into()).collect()),
            MetaVal::Map(m) => Self::Mapping(m.into_iter().map(|(k, v)| (k, v.into())).collect()),
            MetaVal::Bul(b) => Self::Boolean(b),
            MetaVal::Int(i) => Self::Number(Number::Integer(i)),
            MetaVal::Dec(d) => Self::Number(Number::Decimal(d)),
        }
    }
}

impl From<UnaryOp> for StackOperand {
    fn from(unary_op: UnaryOp) -> Self {
        Self::UnaryOp(unary_op)
    }
}

impl From<BinaryOp> for StackOperand {
    fn from(binary_op: BinaryOp) -> Self {
        Self::BinaryOp(binary_op)
    }
}

impl StackOperand {
    const NULL_DESC: &'static str = "null";
    const TEXT_DESC: &'static str = "text";
    const SEQUENCE_DESC: &'static str = "sequence";
    const MAPPING_DESC: &'static str = "mapping";
    const BOOLEAN_DESC: &'static str = "boolean";
    const NUMBER_DESC: &'static str = "number";
    const INTEGER_DESC: &'static str = "integer";
    const DECIMAL_DESC: &'static str = "decimal";
    const UNARY_OP_DESC: &'static str = "unary op";
    const BINARY_OP_DESC: &'static str = "binary op";

    fn description(&self) -> &'static str {
        match &self {
            &Self::Null => Self::NULL_DESC,
            &Self::Text(..) => Self::TEXT_DESC,
            &Self::Sequence(..) => Self::SEQUENCE_DESC,
            &Self::Mapping(..) => Self::MAPPING_DESC,
            &Self::Boolean(..) => Self::BOOLEAN_DESC,
            &Self::Number(Number::Integer(..)) => Self::INTEGER_DESC,
            &Self::Number(Number::Decimal(..)) => Self::DECIMAL_DESC,
            &Self::UnaryOp(..) => Self::UNARY_OP_DESC,
            &Self::BinaryOp(..) => Self::BINARY_OP_DESC,
        }
    }

    fn process_stack_any(stack: &mut Vec<Self>) -> Result<Self, Error> {
        stack.pop().ok_or_else(|| Error::EmptyStack)
    }

    fn process_stack_text(stack: &mut Vec<Self>) -> Result<String, Error> {
        match Self::process_stack_any(stack)? {
            Self::Text(val) => Ok(val),
            other => Err(Error::UnexpectedType{expected: Self::TEXT_DESC, found: other.description()})
        }
    }

    fn process_stack_sequence(stack: &mut Vec<Self>) -> Result<Vec<Self>, Error> {
        match Self::process_stack_any(stack)? {
            Self::Sequence(val) => Ok(val),
            other => Err(Error::UnexpectedType{expected: Self::SEQUENCE_DESC, found: other.description()})
        }
    }

    fn process_stack_mapping(stack: &mut Vec<Self>) -> Result<BTreeMap<MetaKey, Self>, Error> {
        match Self::process_stack_any(stack)? {
            Self::Mapping(val) => Ok(val),
            other => Err(Error::UnexpectedType{expected: Self::MAPPING_DESC, found: other.description()})
        }
    }

    fn process_stack_boolean(stack: &mut Vec<Self>) -> Result<bool, Error> {
        match Self::process_stack_any(stack)? {
            Self::Boolean(val) => Ok(val),
            other => Err(Error::UnexpectedType{expected: Self::BOOLEAN_DESC, found: other.description()})
        }
    }

    fn process_stack_number(stack: &mut Vec<Self>) -> Result<Number, Error> {
        match Self::process_stack_any(stack)? {
            Self::Number(val) => Ok(val),
            other => Err(Error::UnexpectedType{expected: Self::NUMBER_DESC, found: other.description()})
        }
    }

    fn process_stack_integer(stack: &mut Vec<Self>) -> Result<i64, Error> {
        match Self::process_stack_any(stack)? {
            Self::Number(Number::Integer(val)) => Ok(val),
            other => Err(Error::UnexpectedType{expected: Self::INTEGER_DESC, found: other.description()})
        }
    }

    fn process_stack_decimal(stack: &mut Vec<Self>) -> Result<BigDecimal, Error> {
        match Self::process_stack_any(stack)? {
            Self::Number(Number::Decimal(val)) => Ok(val),
            other => Err(Error::UnexpectedType{expected: Self::DECIMAL_DESC, found: other.description()})
        }
    }

    fn process_stack_unary_op(stack: &mut Vec<Self>) -> Result<UnaryOp, Error> {
        match Self::process_stack_any(stack)? {
            Self::UnaryOp(val) => Ok(val),
            other => Err(Error::UnexpectedType{expected: Self::UNARY_OP_DESC, found: other.description()})
        }
    }

    fn process_stack_binary_op(stack: &mut Vec<Self>) -> Result<BinaryOp, Error> {
        match Self::process_stack_any(stack)? {
            Self::BinaryOp(val) => Ok(val),
            other => Err(Error::UnexpectedType{expected: Self::BINARY_OP_DESC, found: other.description()})
        }
    }
}

trait Op {
    fn operate(&self, stack: &mut Vec<StackOperand>) -> Result<StackOperand, Error>;

    fn process(&self, stack: &mut Vec<StackOperand>) -> Result<(), Error> {
        let output = self.operate(stack)?;
        stack.push(output);
        Ok(())
    }
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

// impl UnaryOp {
//     pub fn input_type_spec(&self) -> ParamType {
//         match *self {
//             Self::Count => ParamType::Sequence,
//             Self::First => ParamType::Sequence,
//             Self::Last => ParamType::Sequence,
//             Self::Enum => ParamType::Sequence,
//             Self::Flatten => ParamType::Sequence,
//             Self::FlattenRec => ParamType::Sequence,
//             Self::Max => ParamType::Sequence,
//             Self::Min => ParamType::Sequence,
//             Self::Rev => ParamType::Sequence,
//             Self::Sum => ParamType::Sequence,
//             Self::Product => ParamType::Sequence,
//             Self::Dedup => ParamType::Sequence,
//             Self::Unique => ParamType::Sequence,
//             Self::AllEqual => ParamType::Sequence,
//             Self::Sort => ParamType::Sequence,
//         }
//     }
// }

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

// impl BinaryOp {
//     pub fn input_type_spec(&self) -> (ParamType, ParamType) {
//         match *self {
//             Self::Eq => (ParamType::Any, ParamType::Any),
//             Self::Ne => (ParamType::Any, ParamType::Any),
//             Self::Gt => (ParamType::Any, ParamType::Any),
//             Self::Ge => (ParamType::Any, ParamType::Any),
//             Self::Lt => (ParamType::Any, ParamType::Any),
//             Self::Le => (ParamType::Any, ParamType::Any),
//             Self::Nth => (ParamType::Sequence, ParamType::NNInteger),
//             Self::StepBy => (ParamType::Sequence, ParamType::PosInteger),
//             Self::Chain => (ParamType::Sequence, ParamType::Sequence),
//             Self::Zip => (ParamType::Sequence, ParamType::Sequence),
//             Self::Map => (ParamType::Sequence, ParamType::UnaryOp),
//             Self::Filter => (ParamType::Sequence, ParamType::UnaryOp),
//             Self::SkipWhile => (ParamType::Sequence, ParamType::UnaryOp),
//             Self::TakeWhile => (ParamType::Sequence, ParamType::UnaryOp),
//             Self::Skip => (ParamType::Sequence, ParamType::PosInteger),
//             Self::Take => (ParamType::Sequence, ParamType::PosInteger),
//             Self::Fold => (ParamType::Sequence, ParamType::BinaryOp),
//             Self::All => (ParamType::Sequence, ParamType::UnaryOp),
//             Self::Any => (ParamType::Sequence, ParamType::UnaryOp),
//             Self::Find => (ParamType::Sequence, ParamType::UnaryOp),
//             Self::Position => (ParamType::Sequence, ParamType::UnaryOp),
//             Self::Interleave => (ParamType::Sequence, ParamType::Any),
//             Self::Intersperse => (ParamType::Sequence, ParamType::Sequence),
//             Self::Chunks => (ParamType::Sequence, ParamType::PosInteger),
//             Self::Windows => (ParamType::Sequence, ParamType::PosInteger),
//             Self::Merge => (ParamType::Sequence, ParamType::Sequence),
//         }
//     }
// }

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum TernaryOp {
    Pad,
}

// impl TernaryOp {
//     pub fn input_type_spec(&self) -> (ParamType, ParamType, ParamType) {
//         match *self {
//             Self::Pad => (ParamType::Sequence, ParamType::PosInteger, ParamType::Any),
//         }
//     }
// }
