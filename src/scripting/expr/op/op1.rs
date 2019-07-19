use std::convert::TryInto;
use std::convert::TryFrom;

use crate::metadata::types::MetaVal;
use crate::scripting::Error;
use crate::scripting::expr::Expr;
use crate::scripting::expr::arg::Arg;
use crate::scripting::util::iterable_like::IterableLike;
use crate::scripting::util::ref_iterable_like::RefIterableLike;
use crate::util::Number;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[derive(EnumString)]
#[strum(serialize_all = "snake_case")]
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
    pub fn process<'a>(&self, expr: Expr<'a>) -> Result<Arg<'a>, Error> {
        match self {
            &Self::Collect =>
                IterableLike::try_from(expr)?.collect().map(Arg::from),
            &Self::Count =>
                IterableLike::try_from(expr)?.count().map(Arg::from),
            &Self::First =>
                IterableLike::try_from(expr)?.first().map(Arg::from),
            &Self::Last =>
                IterableLike::try_from(expr)?.last().map(Arg::from),
            &Self::MinIn =>
                IterableLike::try_from(expr)?.min_in().map(Arg::from),
            &Self::MaxIn =>
                IterableLike::try_from(expr)?.max_in().map(Arg::from),
            &Self::Rev =>
                IterableLike::try_from(expr)?.rev().map(Arg::from),
            &Self::Sort =>
                IterableLike::try_from(expr)?.sort().map(Arg::from),
            &Self::Sum =>
                IterableLike::try_from(expr)?.sum().map(Arg::from),
            &Self::Prod =>
                IterableLike::try_from(expr)?.prod().map(Arg::from),
            &Self::AllEqual => {
                match expr.try_into()? {
                    Arg::Value(MetaVal::Seq(ref s)) => RefIterableLike::from(s),
                    Arg::Producer(p) => RefIterableLike::from(p),
                    _ => Err(Error::NotIterable)?,
                }.all_equal().map(Arg::from)
            },
            &Self::Flatten =>
                IterableLike::try_from(expr)?.flatten().map(Arg::from),
            &Self::Dedup =>
                IterableLike::try_from(expr)?.dedup().map(Arg::from),
            &Self::Unique =>
                IterableLike::try_from(expr)?.unique().map(Arg::from),
            &Self::Neg => Ok(Self::neg(expr.try_into()?).into()),
            &Self::Abs => Ok(Self::abs(expr.try_into()?).into()),
            &Self::Not => Ok(Self::not(expr.try_into()?).into()),
        }
    }

    fn neg(number: Number) -> Number {
        match number {
            Number::Integer(i) => Number::Integer(-i),
            Number::Decimal(d) => Number::Decimal(if d == dec!(0) { d } else { -d }),
        }
    }

    fn abs(number: Number) -> Number {
        match number {
            Number::Integer(i) => Number::Integer(i.abs()),
            Number::Decimal(d) => Number::Decimal(d.abs()),
        }
    }

    fn not(b: bool) -> bool {
        !b
    }
}
