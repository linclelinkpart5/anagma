use std::convert::TryInto;
use std::convert::TryFrom;

use crate::metadata::types::MetaVal;
use crate::functions::Error;
use crate::functions::operand::Operand;
use crate::functions::util::iterable_like::IterableLike;
use crate::functions::util::number_like::NumberLike;

#[derive(Clone, Copy, Debug)]
pub enum Op {
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
}

impl Op {
    pub fn process<'a>(&self, o: Operand<'a>) -> Result<Operand<'a>, Error> {
        match self {
            &Self::Collect =>
                IterableLike::try_from(o)?.collect().map(Operand::from),
            &Self::Count =>
                IterableLike::try_from(o)?.count().map(Operand::from),
            &Self::First =>
                IterableLike::try_from(o)?.first().map(Operand::from),
            &Self::Last =>
                IterableLike::try_from(o)?.last().map(Operand::from),
            &Self::MinIn =>
                IterableLike::try_from(o)?.min_in().map(Operand::from),
            &Self::MaxIn =>
                IterableLike::try_from(o)?.max_in().map(Operand::from),
            &Self::Rev =>
                IterableLike::try_from(o)?.rev().map(Operand::from),
            &Self::Sort =>
                IterableLike::try_from(o)?.sort().map(Operand::from),
            &Self::Sum =>
                IterableLike::try_from(o)?.sum().map(Operand::from),
            &Self::Prod =>
                IterableLike::try_from(o)?.prod().map(Operand::from),
            &Self::AllEqual =>
                IterableLike::try_from(o)?.all_equal().map(Operand::from),
            &Self::Flatten =>
                IterableLike::try_from(o)?.flatten().map(Operand::from),
            &Self::Dedup =>
                IterableLike::try_from(o)?.dedup().map(Operand::from),
            &Self::Unique =>
                IterableLike::try_from(o)?.unique().map(Operand::from),
            &Self::Neg => Ok(Self::neg(o.try_into()?).into()),
            &Self::Abs => Ok(Self::abs(o.try_into()?).into()),
            &Self::Not => Ok(Self::not(o.try_into()?).into()),
        }
    }

    fn neg(number: NumberLike) -> NumberLike {
        match number {
            NumberLike::Integer(i) => NumberLike::Integer(-i),
            NumberLike::Decimal(d) => NumberLike::Decimal(if d == dec!(0) { d } else { -d }),
        }
    }

    fn abs(number: NumberLike) -> NumberLike {
        match number {
            NumberLike::Integer(i) => NumberLike::Integer(i.abs()),
            NumberLike::Decimal(d) => NumberLike::Decimal(d.abs()),
        }
    }

    fn not(b: bool) -> bool {
        !b
    }
}
