use std::convert::TryInto;

use metadata::resolver::streams::Stream;
use metadata::types::MetaVal;
use metadata::types::MetaKey;
use metadata::types::MetaKeyPath;
use metadata::resolver::iterable_like::IterableLike;
use metadata::resolver::number_like::NumberLike;
use metadata::resolver::context::ResolverContext;
use metadata::resolver::Error;
use metadata::stream::block::FileMetaBlockStream;
use metadata::stream::value::MetaValueStream;
use util::file_walkers::ParentFileWalker;
use util::file_walkers::ChildFileWalker;

/// Values that are pushed onto an operand stack.
/// In order for a stack to be valid, it must result in exactly one value operand after processing.
pub enum Operand<'k, 'p, 's> {
    Stream(Stream<'k, 'p, 's>),
    Value(MetaVal),
}

pub struct OperandStack<'k, 'p, 's>(Vec<Operand<'k, 'p, 's>>);

impl<'k, 'p, 's> OperandStack<'k, 'p, 's> {
    pub fn pop(&mut self) -> Result<Operand, Error> {
        self.0.pop().ok_or_else(|| Error::EmptyStack)
    }

    pub fn push(&mut self, op: Operand<'k, 'p, 's>) -> () {
        self.0.push(op)
    }

    pub fn pop_iterable_like(&mut self) -> Result<IterableLike, Error> {
        match self.pop()? {
            Operand::Stream(s) => Ok(IterableLike::Stream(s)),
            Operand::Value(MetaVal::Seq(s)) => Ok(IterableLike::Sequence(s)),
            _ => Err(Error::UnexpectedOperand),
        }
    }

    pub fn pop_number_like(&mut self) -> Result<NumberLike, Error> {
        match self.pop()? {
            Operand::Value(MetaVal::Int(i)) => Ok(NumberLike::Integer(i)),
            Operand::Value(MetaVal::Dec(d)) => Ok(NumberLike::Decimal(d)),
            _ => Err(Error::UnexpectedOperand),
        }
    }

    pub fn pop_key_path_like(&mut self) -> Result<MetaKeyPath, Error> {
        let it_like = match self.pop()? {
            Operand::Stream(s) => IterableLike::Stream(s),
            Operand::Value(MetaVal::Seq(s)) => IterableLike::Sequence(s),
            Operand::Value(MetaVal::Str(s)) => {
                // Special case, handle and return.
                return Ok(s.into());
            },
            _ => {
                return Err(Error::UnexpectedOperand);
            }
        };

        let mut mks: Vec<MetaKey> = vec![];

        for mv in it_like.into_iter() {
            match mv? {
                MetaVal::Str(s) => {
                    mks.push(s.into());
                },
                _ => return Err(Error::NotString),
            }
        }

        Ok(mks.into())
    }
}

pub enum Token<'k, 'p, 's> {
    Operand(Operand<'k, 'p, 's>),
    NullaryOp(NullaryOp),
    UnaryOp(UnaryOp),
    BinaryOp,
}

pub trait Op {
    fn process<'k, 'p, 's>(&self, rc: &ResolverContext<'k, 'p, 's>, stack: &mut OperandStack<'k, 'p, 's>) -> Result<(), Error>;
}

#[derive(Clone, Copy, Debug)]
pub enum NullaryOp {
    // () -> Stream<V>
    Parents,
    // () -> Stream<V>
    Children,
}

impl Op for NullaryOp {
    fn process<'k, 'p, 's>(&self, rc: &ResolverContext<'k, 'p, 's>, stack: &mut OperandStack<'k, 'p, 's>) -> Result<(), Error> {
        let mb_stream = match self {
            &Self::Parents => FileMetaBlockStream::new(ParentFileWalker::new(rc.current_item_file_path), rc.meta_format, rc.selection, rc.sort_order),
            &Self::Children => FileMetaBlockStream::new(ChildFileWalker::new(rc.current_item_file_path), rc.meta_format, rc.selection, rc.sort_order),
        };

        let stream = Stream::Raw(MetaValueStream::new(rc.current_key_path.clone(), mb_stream));

        stack.push(Operand::Stream(stream));

        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub enum UnaryOp {
    // (Iterable<V>) -> Sequence<V>
    Collect,
    // (Iterable<V>) -> Integer
    Count,
    // (Iterable<V>) -> V
    First,
    // (Iterable<V>) -> V
    Last,
    // (Iterable<Number>) -> Number
    Max,
    // (Iterable<Number>) -> Number
    Min,
    // (Iterable<V>) -> Sequence<V>
    Rev,
    // (Iterable<Number>) -> Number
    Sum,
    // (Iterable<Number>) -> Number
    Product,
    // (Iterable<V>) -> Boolean
    AllEqual,
    // (Iterable<V>) -> Sequence<V>
    Sort,
}

impl Op for UnaryOp {
    fn process<'k, 'p, 's>(&self, _rc: &ResolverContext<'k, 'p, 's>, stack: &mut OperandStack<'k, 'p, 's>) -> Result<(), Error> {
        let output_operand = match self {
            &Self::Collect | &Self::Rev | &Self::Sort => {
                let mut coll = match stack.pop_iterable_like()? {
                    IterableLike::Stream(st) => st.collect::<Result<Vec<_>, _>>()?,
                    IterableLike::Sequence(sq) => sq,
                };

                match self {
                    &Self::Rev => { coll.reverse(); },
                    // TODO: How do sorting maps work?
                    &Self::Sort => { coll.sort(); },
                    _ => {},
                }

                Operand::Value(MetaVal::Seq(coll))
            },
            &Self::Count => {
                let len = match stack.pop_iterable_like()? {
                    // TODO: Make this work without needing to allocate a vector.
                    IterableLike::Stream(st) => st.collect::<Result<Vec<_>, _>>()?.len() as i64,
                    IterableLike::Sequence(sq) => sq.len() as i64,
                };

                Operand::Value(MetaVal::Int(len))
            },
            &Self::First => {
                let mv = stack.pop_iterable_like()?.into_iter().next().unwrap_or(Ok(MetaVal::Nil))?;
                Operand::Value(mv)
            },
            &Self::Last => {
                let mv = match stack.pop_iterable_like()? {
                    IterableLike::Stream(st) => {
                        let mut last_seen = None;
                        for res_mv in st {
                            last_seen = Some(res_mv?);
                        }

                        last_seen
                    },
                    IterableLike::Sequence(sq) => sq.into_iter().last(),
                }.unwrap_or(MetaVal::Nil);

                Operand::Value(mv)
            },
            &Self::Max => {
                let mut m: Option<NumberLike> = None;

                for mv in stack.pop_iterable_like()? {
                    let num: NumberLike = mv?.try_into()?;

                    m = Some(
                        match m {
                            None => num,
                            Some(curr_m) => curr_m.max(num),
                        }
                    );
                }

                Operand::Value(m.ok_or(Error::EmptyIterable)?.into())
            },
            &Self::Min => {
                let mut m: Option<NumberLike> = None;

                for mv in stack.pop_iterable_like()? {
                    let num: NumberLike = mv?.try_into()?;

                    m = Some(
                        match m {
                            None => num,
                            Some(curr_m) => curr_m.min(num),
                        }
                    );
                }

                Operand::Value(m.ok_or(Error::EmptyIterable)?.into())
            },
            &Self::Sum => {
                let mut total = NumberLike::Integer(0);

                for mv in stack.pop_iterable_like()? {
                    let num: NumberLike = mv?.try_into()?;
                    total += num;
                }

                Operand::Value(total.into())
            },
            &Self::Product => {
                let mut total = NumberLike::Integer(1);

                for mv in stack.pop_iterable_like()? {
                    let num: NumberLike = mv?.try_into()?;
                    total *= num;
                }

                Operand::Value(total.into())
            },
            &Self::AllEqual => {
                let mut it = stack.pop_iterable_like()?.into_iter();

                let res = match it.next() {
                    None => true,
                    Some(res_first) => {
                        let first = res_first?;
                        let mut eq_so_far = true;

                        for res_mv in it {
                            let mv = res_mv?;
                            if mv != first {
                                eq_so_far = false;
                                break;
                            }
                        }

                        eq_so_far
                    }
                };

                Operand::Value(MetaVal::Bul(res))
            },
        };

        stack.push(output_operand);

        Ok(())
    }
}

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
