use std::convert::TryInto;
use std::convert::TryFrom;

use crate::metadata::types::MetaVal;
use crate::functions::Error;
use crate::functions::expr::arg::Arg;
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
    pub fn process<'a>(&self, o: Arg<'a>) -> Result<Arg<'a>, Error> {
        match self {
            &Self::Collect =>
                IterableLike::try_from(o)?.collect().map(Arg::from),
            &Self::Count =>
                IterableLike::try_from(o)?.count().map(Arg::from),
            &Self::First =>
                IterableLike::try_from(o)?.first().map(Arg::from),
            &Self::Last =>
                IterableLike::try_from(o)?.last().map(Arg::from),
            &Self::MinIn =>
                IterableLike::try_from(o)?.min_in().map(Arg::from),
            &Self::MaxIn =>
                IterableLike::try_from(o)?.max_in().map(Arg::from),
            &Self::Rev =>
                IterableLike::try_from(o)?.rev().map(Arg::from),
            &Self::Sort =>
                IterableLike::try_from(o)?.sort().map(Arg::from),
            &Self::Sum =>
                IterableLike::try_from(o)?.sum().map(Arg::from),
            &Self::Prod =>
                IterableLike::try_from(o)?.prod().map(Arg::from),
            &Self::AllEqual =>
                IterableLike::try_from(o)?.all_equal().map(Arg::from),
            &Self::Flatten =>
                IterableLike::try_from(o)?.flatten().map(Arg::from),
            &Self::Dedup =>
                IterableLike::try_from(o)?.dedup().map(Arg::from),
            &Self::Unique =>
                IterableLike::try_from(o)?.unique().map(Arg::from),
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
