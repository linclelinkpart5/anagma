///! Wrapper type for items on the consumer stack that behave as a logical numerical value.

use std::ops::AddAssign;
use std::ops::MulAssign;
use std::borrow::Cow;
use std::convert::TryFrom;

use bigdecimal::BigDecimal;

use crate::metadata::types::MetaVal;
use crate::functions::op::operand::Operand;
use crate::functions::Error;

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

impl<'o> From<NumberLike> for Operand<'o> {
    fn from(nl: NumberLike) -> Self {
        Self::Value(nl.into())
    }
}

impl<'k> From<NumberLike> for MetaVal<'k> {
    fn from(nl: NumberLike) -> MetaVal<'k> {
        match nl {
            NumberLike::Integer(i) => Self::Int(i),
            NumberLike::Decimal(d) => Self::Dec(d),
        }
    }
}
impl<'k> TryFrom<Operand<'k>> for NumberLike {
    type Error = Error;

    fn try_from(value: Operand<'k>) -> Result<Self, Self::Error> {
        match value {
            Operand::Value(mv) => Self::try_from(mv),
            _ => Err(Error::NotNumeric),
        }
    }
}

impl<'k> TryFrom<MetaVal<'k>> for NumberLike {
    type Error = Error;

    fn try_from(value: MetaVal<'k>) -> Result<Self, Self::Error> {
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

impl AddAssign for NumberLike {
    fn add_assign(&mut self, other: Self) {
        *self = match (&self, other) {
            (&Self::Integer(ref l), Self::Integer(r)) => Self::Integer(l + r),
            (&Self::Integer(ref l), Self::Decimal(ref r)) => Self::Decimal(BigDecimal::from(*l) + r),
            (&Self::Decimal(ref l), Self::Integer(r)) => Self::Decimal(l + BigDecimal::from(r)),
            (&Self::Decimal(ref l), Self::Decimal(ref r)) => Self::Decimal(l + r),
        };
    }
}

impl MulAssign for NumberLike {
    fn mul_assign(&mut self, other: Self) {
        *self = match (&self, other) {
            (&Self::Integer(ref l), Self::Integer(r)) => Self::Integer(l * r),
            (&Self::Integer(ref l), Self::Decimal(ref r)) => Self::Decimal(BigDecimal::from(*l) * r),
            (&Self::Decimal(ref l), Self::Integer(r)) => Self::Decimal(l * BigDecimal::from(r)),
            (&Self::Decimal(ref l), Self::Decimal(ref r)) => Self::Decimal(l * r),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::NumberLike;

    #[test]
    fn test_cmp() {
        for l in -3..=3 {
            for r in -3..=3 {
                let li = NumberLike::Integer(l);
                let ld = NumberLike::Decimal(l.into());
                let ri = NumberLike::Integer(r);
                let rd = NumberLike::Decimal(r.into());

                let expected = l.cmp(&r);

                assert_eq!(expected, li.cmp(&ri));
                assert_eq!(expected, li.cmp(&rd));
                assert_eq!(expected, ld.cmp(&ri));
                assert_eq!(expected, ld.cmp(&rd));
            }
        }
    }
}
