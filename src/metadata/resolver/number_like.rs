///! Wrapper type for items on the consumer stack that behave as a logical numerical value.

use std::convert::TryFrom;

use bigdecimal::BigDecimal;

use metadata::resolver::Error;
use metadata::resolver::ops::Operand;
use metadata::types::MetaVal;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum NumberLike {
    Integer(i64),
    Decimal(BigDecimal),
}

impl PartialOrd for NumberLike {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NumberLike {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Self::Integer(l), Self::Integer(r)) => l.cmp(r),
            (Self::Integer(l), Self::Decimal(r)) => BigDecimal::from(*l).cmp(r),
            (Self::Decimal(l), Self::Integer(r)) => l.cmp(&BigDecimal::from(*r)),
            (Self::Decimal(l), Self::Decimal(r)) => l.cmp(r),
        }
    }
}

impl Into<MetaVal> for NumberLike {
    fn into(self) -> MetaVal {
        match self {
            Self::Integer(i) => MetaVal::Int(i),
            Self::Decimal(d) => MetaVal::Dec(d),
        }
    }
}

impl TryFrom<MetaVal> for NumberLike {
    type Error = Error;

    fn try_from(value: MetaVal) -> Result<Self, Self::Error> {
        match value {
            MetaVal::Int(i) => Ok(Self::Integer(i)),
            MetaVal::Dec(d) => Ok(Self::Decimal(d)),
            _ => Err(Error::NotNumeric),
        }
    }
}

impl From<i64> for NumberLike {
    fn from(n: i64) -> Self {
        Self::Integer(n)
    }
}

impl From<BigDecimal> for NumberLike {
    fn from(n: BigDecimal) -> Self {
        Self::Decimal(n)
    }
}

impl<'k, 'p, 's> From<NumberLike> for Operand<'k, 'p, 's> {
    fn from(il: NumberLike) -> Self {
        match il {
            NumberLike::Integer(i) => Self::Value(MetaVal::Int(i)),
            NumberLike::Decimal(d) => Self::Value(MetaVal::Dec(d)),
        }
    }
}
