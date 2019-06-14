use std::convert::TryInto;
use std::convert::TryFrom;
use std::cmp::Ordering;

use crate::metadata::types::MetaVal;
use crate::functions::Error;
use crate::functions::expr::arg::Arg;
use crate::functions::util::iterable_like::IterableLike;
use crate::functions::util::number_like::NumberLike;
// use crate::functions::util::value_producer::ValueProducer;

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
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl Op {
    pub fn process<'a>(&self, o_a: Arg<'a>, o_b: Arg<'a>) -> Result<Arg<'a>, Error> {
        match self {
            &Self::Nth =>
                IterableLike::try_from(o_a)?.nth(o_b.try_into()?).map(Arg::from),
            &Self::All =>
                IterableLike::try_from(o_a)?.all(o_b.try_into()?).map(Arg::from),
            &Self::Any =>
                IterableLike::try_from(o_a)?.any(o_b.try_into()?).map(Arg::from),
            &Self::Find =>
                IterableLike::try_from(o_a)?.find(o_b.try_into()?).map(Arg::from),
            &Self::Position =>
                IterableLike::try_from(o_a)?.position(o_b.try_into()?).map(Arg::from),
            &Self::Filter =>
                IterableLike::try_from(o_a)?.filter(o_b.try_into()?).map(Arg::from),
            &Self::Map =>
                IterableLike::try_from(o_a)?.map(o_b.try_into()?).map(Arg::from),
            &Self::StepBy =>
                IterableLike::try_from(o_a)?.step_by(o_b.try_into()?).map(Arg::from),
            &Self::Chain =>
                IterableLike::try_from(o_a)?.chain(o_b.try_into()?).map(Arg::from),
            &Self::Zip =>
                IterableLike::try_from(o_a)?.zip(o_b.try_into()?).map(Arg::from),
            &Self::Skip =>
                IterableLike::try_from(o_a)?.skip(o_b.try_into()?).map(Arg::from),
            &Self::Take =>
                IterableLike::try_from(o_a)?.take(o_b.try_into()?).map(Arg::from),
            &Self::SkipWhile =>
                IterableLike::try_from(o_a)?.skip_while(o_b.try_into()?).map(Arg::from),
            &Self::TakeWhile =>
                IterableLike::try_from(o_a)?.take_while(o_b.try_into()?).map(Arg::from),
            _ => Ok(Arg::Value(MetaVal::Nil)),
        }
    }

    fn eq(mv_a: &MetaVal, mv_b: &MetaVal) -> bool {
        mv_a == mv_b
    }

    fn ne(mv_a: &MetaVal, mv_b: &MetaVal) -> bool {
        mv_a != mv_b
    }

    fn lt(num_a: &NumberLike, num_b: &NumberLike) -> Result<bool, Error> {
        let ord = num_a.val_cmp(&num_b);
        Ok(ord == Ordering::Less)
    }

    fn le(num_a: &NumberLike, num_b: &NumberLike) -> Result<bool, Error> {
        let ord = num_a.val_cmp(&num_b);
        Ok(ord == Ordering::Less || ord == Ordering::Equal)
    }

    fn gt(num_a: &NumberLike, num_b: &NumberLike) -> Result<bool, Error> {
        let ord = num_a.val_cmp(&num_b);
        Ok(ord == Ordering::Greater)
    }

    fn ge(num_a: &NumberLike, num_b: &NumberLike) -> Result<bool, Error> {
        let ord = num_a.val_cmp(&num_b);
        Ok(ord == Ordering::Greater || ord == Ordering::Equal)
    }
}
