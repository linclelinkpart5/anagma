use std::convert::TryInto;

use crate::metadata::types::MetaVal;
use crate::metadata::resolver::Error;
use crate::metadata::resolver::ops::Op;
use crate::metadata::resolver::ops::Operand;
use crate::metadata::resolver::ops::OperandStack;
use crate::metadata::resolver::context::ResolverContext;

use crate::metadata::resolver::number_like::NumberLike;
use crate::metadata::resolver::iterable_like::IterableLike;

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
    MaxIn,
    // (Iterable<Number>) -> Number
    MinIn,
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

impl UnaryOp {
    pub fn process<'o>(&self, operand: Operand<'o>) -> Result<Operand<'o>, Error> {
        Ok(match self {
            &Self::Collect | &Self::Rev | &Self::Sort => {
                let mut coll = match operand.try_into()? {
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
                let len = match operand.try_into()? {
                    // TODO: Make this work without needing to allocate a vector.
                    IterableLike::Stream(st) => st.collect::<Result<Vec<_>, _>>()?.len() as i64,
                    IterableLike::Sequence(sq) => sq.len() as i64,
                };

                Operand::Value(MetaVal::Int(len))
            },
            &Self::First => {
                // LEARN: Why is a turbofish not allowed here?
                // let mv = operand.try_into()?.into_iter().next().unwrap_or(Ok(MetaVal::Nil))?;
                let il: IterableLike<'_> = operand.try_into()?;
                let mv = il.into_iter().next().unwrap_or(Ok(MetaVal::Nil))?;
                Operand::Value(mv)
            },
            &Self::Last => {
                let mv = match operand.try_into()? {
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
            &Self::MaxIn | &Self::MinIn => {
                let mut m: Option<NumberLike> = None;

                let il: IterableLike<'_> = operand.try_into()?;

                for mv in il {
                    let num: NumberLike = mv?.try_into()?;

                    m = Some(
                        match m {
                            None => num,
                            Some(curr_m) => {
                                match self {
                                    &Self::MaxIn => curr_m.max(num),
                                    &Self::MinIn => curr_m.min(num),
                                    _ => unreachable!(),
                                }
                            },
                        }
                    );
                }

                Operand::Value(m.ok_or(Error::EmptyIterable)?.into())
            },
            &Self::Sum | &Self::Product => {
                let mut total = match self {
                    &Self::Sum => NumberLike::Integer(0),
                    &Self::Product => NumberLike::Integer(1),
                    _ => unreachable!(),
                };

                let il: IterableLike<'_> = operand.try_into()?;

                for mv in il {
                    let num: NumberLike = mv?.try_into()?;
                    match self {
                        &Self::Sum => { total += num; },
                        &Self::Product => { total *= num; },
                        _ => unreachable!(),
                    };
                }

                Operand::Value(total.into())
            },
            &Self::AllEqual => {
                let il: IterableLike<'_> = operand.try_into()?;
                let mut it = il.into_iter();

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
        })
    }
}

impl Op for UnaryOp {
    fn process_stack<'bo>(&self, stack: &mut OperandStack<'bo>) -> Result<(), Error> {
        let input = stack.pop()?;
        let output = self.process(input)?;

        stack.push(output);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::UnaryOp;

    use bigdecimal::BigDecimal;

    use crate::metadata::resolver::ops::Op;
    use crate::metadata::resolver::ops::Operand;
    use crate::metadata::resolver::ops::OperandStack;
    use crate::metadata::resolver::streams::Stream;

    use crate::metadata::types::MetaVal;

    use crate::test_util::TestUtil;

    fn stackify_meta_vals<'a, II>(mvs: II) -> OperandStack<'a>
    where
        II: IntoIterator<Item = MetaVal<'a>>,
    {
        let fmvs = TestUtil::create_fixed_value_stream(mvs);

        let mut stack = OperandStack::new();
        stack.push(Operand::Stream(Stream::Raw(fmvs.into())));
        stack
    }

    #[test]
    fn test_process() {
        let mut stack = stackify_meta_vals(vec![
            TestUtil::sample_string(),
            TestUtil::sample_integer(),
            TestUtil::sample_boolean(),
            TestUtil::sample_decimal(),
            TestUtil::sample_null(),
        ]);

        UnaryOp::Collect.process_stack(&mut stack).expect("process failed");

        assert_eq!(1, stack.len());
        match stack.pop().expect("stack is empty") {
            Operand::Value(MetaVal::Seq(seq)) => {
                assert_eq!(
                    vec![
                        TestUtil::sample_string(),
                        TestUtil::sample_integer(),
                        TestUtil::sample_boolean(),
                        TestUtil::sample_decimal(),
                        TestUtil::sample_null(),
                    ],
                    seq
                );
            },
            _ => { panic!("unexpected operand"); },
        }

        let mut stack = stackify_meta_vals(vec![
            TestUtil::sample_string(),
            TestUtil::sample_integer(),
            TestUtil::sample_boolean(),
            TestUtil::sample_decimal(),
            TestUtil::sample_null(),
        ]);

        UnaryOp::Rev.process_stack(&mut stack).expect("process failed");

        assert_eq!(1, stack.len());
        match stack.pop().expect("stack is empty") {
            Operand::Value(MetaVal::Seq(seq)) => {
                assert_eq!(
                    vec![
                        TestUtil::sample_null(),
                        TestUtil::sample_decimal(),
                        TestUtil::sample_boolean(),
                        TestUtil::sample_integer(),
                        TestUtil::sample_string(),
                    ],
                    seq
                );
            },
            _ => { panic!("unexpected operand"); },
        }

        let mut stack = stackify_meta_vals(vec![
            TestUtil::sample_string(),
            TestUtil::sample_integer(),
            TestUtil::sample_boolean(),
            TestUtil::sample_decimal(),
            TestUtil::sample_null(),
        ]);

        UnaryOp::Count.process_stack(&mut stack).expect("process failed");

        assert_eq!(1, stack.len());
        match stack.pop().expect("stack is empty") {
            Operand::Value(MetaVal::Int(i)) => { assert_eq!(5, i); },
            _ => { panic!("unexpected operand"); },
        }

        let mut stack = stackify_meta_vals(vec![
            TestUtil::sample_string(),
            TestUtil::sample_integer(),
            TestUtil::sample_boolean(),
            TestUtil::sample_decimal(),
            TestUtil::sample_null(),
        ]);

        UnaryOp::First.process_stack(&mut stack).expect("process failed");

        assert_eq!(1, stack.len());
        match stack.pop().expect("stack is empty") {
            Operand::Value(mv) => { assert_eq!(TestUtil::sample_string(), mv); },
            _ => { panic!("unexpected operand"); },
        }

        let mut stack = stackify_meta_vals(vec![
            TestUtil::sample_string(),
            TestUtil::sample_integer(),
            TestUtil::sample_boolean(),
            TestUtil::sample_decimal(),
            TestUtil::sample_null(),
        ]);

        UnaryOp::Last.process_stack(&mut stack).expect("process failed");

        assert_eq!(1, stack.len());
        match stack.pop().expect("stack is empty") {
            Operand::Value(mv) => { assert_eq!(TestUtil::sample_null(), mv); },
            _ => { panic!("unexpected operand"); },
        }

        let mut stack = stackify_meta_vals(vec![
            MetaVal::Int(-1),
            MetaVal::Int(-2),
            MetaVal::Dec(BigDecimal::new((-15).into(), 1)),
            MetaVal::Dec(BigDecimal::new((-5).into(), 1)),
            MetaVal::Dec(BigDecimal::new(5.into(), 1)),
            MetaVal::Dec(BigDecimal::new(15.into(), 1)),
            MetaVal::Int(2),
            MetaVal::Int(1),
        ]);

        UnaryOp::MaxIn.process_stack(&mut stack).expect("process failed");

        assert_eq!(1, stack.len());
        match stack.pop().expect("stack is empty") {
            Operand::Value(mv) => { assert_eq!(MetaVal::Int(2), mv); },
            _ => { panic!("unexpected operand"); },
        }

        let mut stack = stackify_meta_vals(vec![
            MetaVal::Int(-1),
            MetaVal::Int(-2),
            MetaVal::Dec(BigDecimal::new((-15).into(), 1)),
            MetaVal::Dec(BigDecimal::new((-5).into(), 1)),
            MetaVal::Dec(BigDecimal::new(5.into(), 1)),
            MetaVal::Dec(BigDecimal::new(15.into(), 1)),
            MetaVal::Int(2),
            MetaVal::Int(1),
        ]);

        UnaryOp::MinIn.process_stack(&mut stack).expect("process failed");

        assert_eq!(1, stack.len());
        match stack.pop().expect("stack is empty") {
            Operand::Value(mv) => { assert_eq!(MetaVal::Int(-2), mv); },
            _ => { panic!("unexpected operand"); },
        }

        let mut stack = stackify_meta_vals(vec![
            MetaVal::Int(1),
            MetaVal::Int(2),
            MetaVal::Int(3),
            MetaVal::Int(4),
            MetaVal::Int(5),
        ]);

        UnaryOp::Sum.process_stack(&mut stack).expect("process failed");

        assert_eq!(1, stack.len());
        match stack.pop().expect("stack is empty") {
            Operand::Value(mv) => { assert_eq!(MetaVal::Int(15), mv); },
            _ => { panic!("unexpected operand"); },
        }

        let mut stack = stackify_meta_vals(vec![
            MetaVal::Int(1),
            MetaVal::Int(2),
            MetaVal::Int(3),
            MetaVal::Int(4),
            MetaVal::Dec(BigDecimal::from(5.5)),
        ]);

        UnaryOp::Sum.process_stack(&mut stack).expect("process failed");

        assert_eq!(1, stack.len());
        match stack.pop().expect("stack is empty") {
            Operand::Value(mv) => { assert_eq!(MetaVal::Dec(BigDecimal::from(15.5)), mv); },
            _ => { panic!("unexpected operand"); },
        }

        let mut stack = stackify_meta_vals(vec![
            MetaVal::Int(1),
            MetaVal::Int(2),
            MetaVal::Int(3),
            MetaVal::Int(4),
            MetaVal::Int(5),
        ]);

        UnaryOp::Product.process_stack(&mut stack).expect("process failed");

        assert_eq!(1, stack.len());
        match stack.pop().expect("stack is empty") {
            Operand::Value(mv) => { assert_eq!(MetaVal::Int(120), mv); },
            _ => { panic!("unexpected operand"); },
        }

        let mut stack = stackify_meta_vals(vec![
            MetaVal::Int(1),
            MetaVal::Int(2),
            MetaVal::Int(3),
            MetaVal::Int(4),
            MetaVal::Dec(BigDecimal::from(5.5)),
        ]);

        UnaryOp::Product.process_stack(&mut stack).expect("process failed");

        assert_eq!(1, stack.len());
        match stack.pop().expect("stack is empty") {
            Operand::Value(mv) => { assert_eq!(MetaVal::Dec(BigDecimal::from(132)), mv); },
            _ => { panic!("unexpected operand"); },
        }

        let mut stack = stackify_meta_vals(vec![
            MetaVal::Int(1),
            MetaVal::Int(1),
            MetaVal::Int(1),
        ]);

        UnaryOp::AllEqual.process_stack(&mut stack).expect("process failed");

        assert_eq!(1, stack.len());
        match stack.pop().expect("stack is empty") {
            Operand::Value(mv) => { assert_eq!(MetaVal::Bul(true), mv); },
            _ => { panic!("unexpected operand"); },
        }

        let mut stack = stackify_meta_vals(vec![]);

        UnaryOp::AllEqual.process_stack(&mut stack).expect("process failed");

        assert_eq!(1, stack.len());
        match stack.pop().expect("stack is empty") {
            Operand::Value(mv) => { assert_eq!(MetaVal::Bul(true), mv); },
            _ => { panic!("unexpected operand"); },
        }

        let mut stack = stackify_meta_vals(vec![
            MetaVal::Int(1),
            MetaVal::Int(1),
            MetaVal::Int(-1),
        ]);

        UnaryOp::AllEqual.process_stack(&mut stack).expect("process failed");

        assert_eq!(1, stack.len());
        match stack.pop().expect("stack is empty") {
            Operand::Value(mv) => { assert_eq!(MetaVal::Bul(false), mv); },
            _ => { panic!("unexpected operand"); },
        }
    }
}
