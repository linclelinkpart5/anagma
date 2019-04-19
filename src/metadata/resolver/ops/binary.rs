use std::cmp::Ordering;
use std::convert::TryInto;

use bigdecimal::BigDecimal;

use crate::metadata::types::MetaVal;
use crate::metadata::resolver::Error;
use crate::metadata::resolver::streams::Stream;
use crate::metadata::resolver::streams::StepByStream;
use crate::metadata::resolver::ops::Op;
use crate::metadata::resolver::ops::Operand;
use crate::metadata::resolver::ops::OperandStack;

use crate::metadata::resolver::number_like::NumberLike;
use crate::metadata::resolver::iterable_like::IterableLike;
use crate::metadata::resolver::iterable_like::Index;

#[derive(Clone, Copy, Debug)]
pub enum BinaryOp {
    // (Iterable<V>, Usize) -> V
    Nth,
    // (Stream<V>, Usize) -> Stream<V>
    // (Sequence<V>, Usize) -> Sequence<V>
    StepBy,
    // (Sequence<V>, Sequence<V>) -> Sequence<V>
    // (Stream<V>, Iterable<V>) -> Stream<V>
    // (Iterable<V>, Stream<V>) -> Stream<V>
    Chain,
    // (Sequence<V>, Sequence<V>) -> Sequence<Sequence<V>>
    // (Stream<V>, Iterable<V>) -> Stream<Sequence<V>>
    // (Iterable<V>, Stream<V>) -> Stream<Sequence<V>>
    Zip,
    // (Stream<V>, UnaryOp) -> Stream<V>
    // (Sequence<V>, UnaryOp) -> Sequence<V>
    Map,
    // (Stream<V>, Predicate) -> Stream<V>
    // (Sequence<V>, Predicate) -> Sequence<V>
    Filter,
    // (Stream<V>, Predicate) -> Stream<V>
    // (Sequence<V>, Predicate) -> Sequence<V>
    SkipWhile,
    // (Stream<V>, Predicate) -> Stream<V>
    // (Sequence<V>, Predicate) -> Sequence<V>
    TakeWhile,
    // (Stream<V>, Usize) -> Stream<V>
    // (Sequence<V>, Usize) -> Sequence<V>
    Skip,
    // (Stream<V>, Usize) -> Stream<V>
    // (Sequence<V>, Usize) -> Sequence<V>
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
    // (Stream<V>, Iterable<V>) -> Stream<V>
    // (Iterable<V>, Stream<V>) -> Stream<V>
    Interleave,
    // (Stream<V>, V) -> Stream<V>
    // (Sequence<V>, V) -> Sequence<V>
    Intersperse,
    // (Stream<V>, Usize) -> Stream<Sequence<V>>
    // (Sequence<V>, Usize) -> Sequence<Sequence<V>>
    Chunks,
    // (Stream<V>, Usize) -> Stream<Sequence<V>>
    // (Sequence<V>, Usize) -> Sequence<Sequence<V>>
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

                let (collect_after, stream) = match il {
                    IterableLike::Sequence(s) => (true, Stream::Fixed(s.into_iter())),
                    IterableLike::Stream(s) => (false, s),
                };

                let adapted_stream = Stream::StepBy(StepByStream::new(stream, n)?);

                if collect_after {
                    Operand::Value(MetaVal::Seq(adapted_stream.collect::<Result<Vec<_>, _>>()?))
                }
                else {
                    Operand::Stream(adapted_stream)
                }
            },
            _ => Operand::Value(MetaVal::Nil),
        })
    }
}
