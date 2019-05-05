use std::convert::TryInto;

use crate::metadata::types::MetaVal;
use crate::metadata::resolver::Error;
use crate::metadata::resolver::streams::Stream;
use crate::metadata::resolver::streams::StepByStream;
use crate::metadata::resolver::streams::ChainStream;
use crate::metadata::resolver::streams::ZipStream;
use crate::metadata::resolver::streams::MapStream;
use crate::metadata::resolver::ops::Operand;
use crate::metadata::resolver::ops::unary::UnaryOp;

use crate::metadata::resolver::iterable_like::IterableLike;
use crate::metadata::resolver::iterable_like::Index;

#[derive(Clone, Copy, Debug)]
pub enum BinaryOp {
    // (Iterable<V>, Usize) -> V
    Nth,
    // (Sequence<V>, Usize) -> Sequence<V>
    // (Stream<V>, Usize) -> Stream<V>
    StepBy,
    // (Sequence<V>, Sequence<V>) -> Sequence<V>
    // (Iterable<V>, Iterable<V>) -> Stream<V>
    Chain,
    // (Sequence<V>, Sequence<V>) -> Sequence<Sequence<V>>
    // (Iterable<V>, Iterable<V>) -> Stream<Sequence<V>>
    Zip,
    // (Sequence<V>, UnaryOp) -> Sequence<V>
    // (Stream<V>, UnaryOp) -> Stream<V>
    Map,
    // (Sequence<V>, Predicate) -> Sequence<V>
    // (Stream<V>, Predicate) -> Stream<V>
    Filter,
    // (Sequence<V>, Predicate) -> Sequence<V>
    // (Stream<V>, Predicate) -> Stream<V>
    SkipWhile,
    // (Sequence<V>, Predicate) -> Sequence<V>
    // (Stream<V>, Predicate) -> Stream<V>
    TakeWhile,
    // (Sequence<V>, Usize) -> Sequence<V>
    // (Stream<V>, Usize) -> Stream<V>
    Skip,
    // (Sequence<V>, Usize) -> Sequence<V>
    // (Stream<V>, Usize) -> Stream<V>
    Take,
    // (Iterable<V>, Predicate) -> Boolean
    All,
    // (Iterable<V>, Predicate) -> Boolean
    Any,
    // (Iterable<V>, Predicate) -> V
    Find,
    // (Iterable<V>, Predicate) -> Usize
    Position,
    // (Sequence<V>, Sequence<V>) -> Sequence<V>
    // (Iterable<V>, Iterable<V>) -> Stream<V>
    Interleave,
    // (Sequence<V>, V) -> Sequence<V>
    // (Stream<V>, V) -> Stream<V>
    Intersperse,
    // (Sequence<V>, Usize) -> Sequence<Sequence<V>>
    // (Stream<V>, Usize) -> Stream<Sequence<V>>
    Chunks,
    // (Sequence<V>, Usize) -> Sequence<Sequence<V>>
    // (Stream<V>, Usize) -> Stream<Sequence<V>>
    Windows,
}

impl BinaryOp {
    pub fn process<'o>(&self, operand_a: Operand<'o>, operand_b: Operand<'o>) -> Result<Operand<'o>, Error> {
        Ok(match self {
            &Self::Nth => {
                let il: IterableLike<'_> = operand_a.try_into()?;
                let mut n: Index = operand_b.try_into()?;

                for res_mv in il {
                    let mv = res_mv?;

                    if n == 0 { return Ok(Operand::Value(mv)); }
                    else { n -= 1; }
                }

                return Err(Error::IndexOutOfBounds);
            },
            &Self::StepBy => {
                let il: IterableLike<'_> = operand_a.try_into()?;
                let n: Index = operand_b.try_into()?;

                let collect_after = il.is_eager();
                let adapted_stream = Stream::StepBy(StepByStream::new(il.into(), n)?);

                adapted_stream.into_operand(collect_after)?
            },
            &Self::Chain => {
                let il_a: IterableLike<'_> = operand_a.try_into()?;
                let il_b: IterableLike<'_> = operand_b.try_into()?;

                let collect_after = il_a.is_eager() && il_b.is_eager();

                let adapted_stream = Stream::Chain(ChainStream::new(il_a.into(), il_b.into()));

                adapted_stream.into_operand(collect_after)?
            },
            &Self::Zip => {
                let il_a: IterableLike<'_> = operand_a.try_into()?;
                let il_b: IterableLike<'_> = operand_b.try_into()?;

                let collect_after = il_a.is_eager() && il_b.is_eager();

                let adapted_stream = Stream::Zip(ZipStream::new(il_a.into(), il_b.into()));

                adapted_stream.into_operand(collect_after)?
            },
            &Self::Map => {
                let il: IterableLike<'_> = operand_a.try_into()?;
                let op: UnaryOp = operand_b.try_into()?;

                let collect_after = il.is_eager();

                let adapted_stream = Stream::Map(MapStream::new(il.into(), op));

                adapted_stream.into_operand(collect_after)?
            },
            _ => Operand::Value(MetaVal::Nil),
        })
    }
}
