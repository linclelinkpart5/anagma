use std::convert::TryInto;
use std::convert::TryFrom;
use std::cmp::Ordering;

use crate::metadata::types::MetaVal;
use crate::scripting::Error;
use crate::scripting::expr::Expr;
use crate::scripting::expr::arg::Arg;
use crate::scripting::util::iterable_like::IterableLike;
use crate::scripting::util::ref_iterable_like::RefIterableLike;
use crate::scripting::util::number_like::NumberLike;
// use crate::scripting::util::value_producer::ValueProducer;

#[derive(Clone, Copy, Debug)]
pub enum Op {
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
    // Interleave,
    // Intersperse,
    // Chunks,
    // Windows,
    And,
    Or,
    Xor,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl Op {
    pub fn process<'a>(&self, expr_a: Expr<'a>, expr_b: Expr<'a>) -> Result<Arg<'a>, Error> {
        match self {
            &Self::Nth =>
                IterableLike::try_from(expr_a)?.nth(expr_b.try_into()?).map(Arg::from),
            &Self::All => {
                match expr_a.try_into()? {
                    Arg::Value(MetaVal::Seq(ref s)) => RefIterableLike::from(s),
                    Arg::Producer(p) => RefIterableLike::from(p),
                    _ => Err(Error::NotIterable)?,
                }.all(expr_b.try_into()?).map(Arg::from)
            },
            &Self::Any => {
                match expr_a.try_into()? {
                    Arg::Value(MetaVal::Seq(ref s)) => RefIterableLike::from(s),
                    Arg::Producer(p) => RefIterableLike::from(p),
                    _ => Err(Error::NotIterable)?,
                }.any(expr_b.try_into()?).map(Arg::from)
            },
            &Self::Find =>
                IterableLike::try_from(expr_a)?.find(expr_b.try_into()?).map(Arg::from),
            &Self::Position =>
                IterableLike::try_from(expr_a)?.position(expr_b.try_into()?).map(Arg::from),
            &Self::Filter =>
                IterableLike::try_from(expr_a)?.filter(expr_b.try_into()?).map(Arg::from),
            &Self::Map =>
                IterableLike::try_from(expr_a)?.map(expr_b.try_into()?).map(Arg::from),
            &Self::StepBy =>
                IterableLike::try_from(expr_a)?.step_by(expr_b.try_into()?).map(Arg::from),
            &Self::Chain =>
                IterableLike::try_from(expr_a)?.chain(expr_b.try_into()?).map(Arg::from),
            &Self::Zip =>
                IterableLike::try_from(expr_a)?.zip(expr_b.try_into()?).map(Arg::from),
            &Self::Skip =>
                IterableLike::try_from(expr_a)?.skip(expr_b.try_into()?).map(Arg::from),
            &Self::Take =>
                IterableLike::try_from(expr_a)?.take(expr_b.try_into()?).map(Arg::from),
            &Self::SkipWhile =>
                IterableLike::try_from(expr_a)?.skip_while(expr_b.try_into()?).map(Arg::from),
            &Self::TakeWhile =>
                IterableLike::try_from(expr_a)?.take_while(expr_b.try_into()?).map(Arg::from),
            &Self::And =>
                Self::and(expr_a, expr_b).map(Arg::from),
            &Self::Or =>
                Self::or(expr_a, expr_b).map(Arg::from),
            &Self::Xor =>
                Self::xor(expr_a, expr_b).map(Arg::from),
            &Self::Eq => {
                let num_a = NumberLike::try_from(expr_a)?;
                let num_b = NumberLike::try_from(expr_b)?;
                Ok(Self::eq(&num_a, &num_b).into())
            },
            &Self::Ne => {
                let num_a = NumberLike::try_from(expr_a)?;
                let num_b = NumberLike::try_from(expr_b)?;
                Ok(Self::ne(&num_a, &num_b).into())
            },
            &Self::Lt => {
                let num_a = NumberLike::try_from(expr_a)?;
                let num_b = NumberLike::try_from(expr_b)?;
                Ok(Self::lt(&num_a, &num_b).into())
            },
            &Self::Le => {
                let num_a = NumberLike::try_from(expr_a)?;
                let num_b = NumberLike::try_from(expr_b)?;
                Ok(Self::le(&num_a, &num_b).into())
            },
            &Self::Gt => {
                let num_a = NumberLike::try_from(expr_a)?;
                let num_b = NumberLike::try_from(expr_b)?;
                Ok(Self::gt(&num_a, &num_b).into())
            },
            &Self::Ge => {
                let num_a = NumberLike::try_from(expr_a)?;
                let num_b = NumberLike::try_from(expr_b)?;
                Ok(Self::ge(&num_a, &num_b).into())
            },
        }
    }

    pub fn and(expr_a: Expr, expr_b: Expr) -> Result<bool, Error> {
        Ok(expr_a.try_into()? && expr_b.try_into()?)
    }

    pub fn or(expr_a: Expr, expr_b: Expr) -> Result<bool, Error> {
        Ok(expr_a.try_into()? || expr_b.try_into()?)
    }

    pub fn xor(expr_a: Expr, expr_b: Expr) -> Result<bool, Error> {
        let b_a: bool = expr_a.try_into()?;
        let b_b: bool = expr_b.try_into()?;
        Ok(b_a ^ b_b)
    }

    fn eq(num_a: &NumberLike, num_b: &NumberLike) -> bool {
        let ord = num_a.val_cmp(&num_b);
        ord == Ordering::Equal
    }

    fn ne(num_a: &NumberLike, num_b: &NumberLike) -> bool {
        let ord = num_a.val_cmp(&num_b);
        ord != Ordering::Equal
    }

    fn lt(num_a: &NumberLike, num_b: &NumberLike) -> bool {
        let ord = num_a.val_cmp(&num_b);
        ord == Ordering::Less
    }

    fn le(num_a: &NumberLike, num_b: &NumberLike) -> bool {
        let ord = num_a.val_cmp(&num_b);
        ord == Ordering::Less || ord == Ordering::Equal
    }

    fn gt(num_a: &NumberLike, num_b: &NumberLike) -> bool {
        let ord = num_a.val_cmp(&num_b);
        ord == Ordering::Greater
    }

    fn ge(num_a: &NumberLike, num_b: &NumberLike) -> bool {
        let ord = num_a.val_cmp(&num_b);
        ord == Ordering::Greater || ord == Ordering::Equal
    }
}
