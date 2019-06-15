use std::convert::TryInto;
use std::convert::TryFrom;
// use std::cmp::Ordering;

use crate::metadata::types::MetaVal;
use crate::scripting::Error;
use crate::scripting::expr::Expr;
use crate::scripting::expr::arg::Arg;
use crate::scripting::util::iterable_like::IterableLike;
// use crate::scripting::util::number_like::NumberLike;
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
            &Self::All =>
                IterableLike::try_from(expr_a)?.all(expr_b.try_into()?).map(Arg::from),
            &Self::Any =>
                IterableLike::try_from(expr_a)?.any(expr_b.try_into()?).map(Arg::from),
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
            _ => Ok(Arg::Value(MetaVal::Nil)),
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

    // fn eq(mv_a: &MetaVal, mv_b: &MetaVal) -> bool {
    //     mv_a == mv_b
    // }

    // fn ne(mv_a: &MetaVal, mv_b: &MetaVal) -> bool {
    //     mv_a != mv_b
    // }

    // fn lt(num_a: &NumberLike, num_b: &NumberLike) -> Result<bool, Error> {
    //     let ord = num_a.val_cmp(&num_b);
    //     Ok(ord == Ordering::Less)
    // }

    // fn le(num_a: &NumberLike, num_b: &NumberLike) -> Result<bool, Error> {
    //     let ord = num_a.val_cmp(&num_b);
    //     Ok(ord == Ordering::Less || ord == Ordering::Equal)
    // }

    // fn gt(num_a: &NumberLike, num_b: &NumberLike) -> Result<bool, Error> {
    //     let ord = num_a.val_cmp(&num_b);
    //     Ok(ord == Ordering::Greater)
    // }

    // fn ge(num_a: &NumberLike, num_b: &NumberLike) -> Result<bool, Error> {
    //     let ord = num_a.val_cmp(&num_b);
    //     Ok(ord == Ordering::Greater || ord == Ordering::Equal)
    // }
}
