use std::cmp::Ordering;
use std::convert::TryInto;

use bigdecimal::BigDecimal;

use crate::metadata::types::MetaVal;
use crate::metadata::resolver::Error;
use crate::metadata::resolver::streams::Stream;
use crate::metadata::resolver::streams::FlattenStream;
use crate::metadata::resolver::streams::DedupStream;
use crate::metadata::resolver::streams::UniqueStream;
use crate::metadata::resolver::ops::Op;
use crate::metadata::resolver::ops::Operand;
use crate::metadata::resolver::ops::OperandStack;

use crate::metadata::resolver::number_like::NumberLike;
use crate::metadata::resolver::iterable_like::IterableLike;

fn smart_sort_by<'mv>(a: &MetaVal<'mv>, b: &MetaVal<'mv>) -> Ordering {
    // Smooth over comparsions between integers and decimals.
    match (a, b) {
        (&MetaVal::Int(ref i), &MetaVal::Dec(ref d)) => {
            let i_d: BigDecimal = (*i).into();
            i_d.cmp(&d)
        },
        (&MetaVal::Dec(ref d), &MetaVal::Int(ref i)) => {
            let i_d: BigDecimal = (*i).into();
            d.cmp(&i_d)
        },
        (na, nb) => na.cmp(&nb),
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

    // (Sequence<V>) -> Sequence<V>
    // (Stream<V>) -> Stream<V>
    Flatten,
    // (Sequence<V>) -> Sequence<V>
    // (Stream<V>) -> Stream<V>
    Dedup,
    // (Sequence<V>) -> Sequence<V>
    // (Stream<V>) -> Stream<V>
    Unique,
}

impl UnaryOp {
    pub fn process_as_predicate<'o>(&self, operand: Operand<'o>) -> Result<bool, Error> {
        match self.process(operand)? {
            Operand::Value(MetaVal::Bul(b)) => Ok(b),
            _ => Err(Error::InvalidPredicate),
        }
    }

    pub fn process_as_converter<'o>(&self, operand: Operand<'o>) -> Result<MetaVal<'o>, Error> {
        match self.process(operand)? {
            Operand::Value(mv) => Ok(mv),
            _ => Err(Error::InvalidConverter),
        }
    }

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
                    &Self::Sort => { coll.sort_by(smart_sort_by); },
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
                let il: IterableLike<'_> = operand.try_into()?;
                let mv = il.into_iter().next().unwrap_or(Err(Error::EmptyIterable))?;
                Operand::Value(mv)
            },
            &Self::Last => {
                let opt_mv = match operand.try_into()? {
                    IterableLike::Stream(st) => {
                        let mut last_seen = None;
                        for res_mv in st {
                            last_seen = Some(res_mv?);
                        }

                        last_seen
                    },
                    IterableLike::Sequence(sq) => sq.into_iter().last(),
                };

                match opt_mv {
                    Some(mv) => Operand::Value(mv),
                    None => Err(Error::EmptyIterable)?,
                }
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
            &Self::Flatten | &Self::Dedup | &Self::Unique => {
                let il: IterableLike<'_> = operand.try_into()?;

                let (collect_after, stream) = match il {
                    IterableLike::Sequence(s) => (true, Stream::Fixed(s.into_iter())),
                    IterableLike::Stream(s) => (false, s),
                };

                let adapted_stream = match self {
                    &Self::Flatten => Stream::Flatten(FlattenStream::new(stream)),
                    &Self::Dedup => Stream::Dedup(DedupStream::new(stream)),
                    &Self::Unique => Stream::Unique(UniqueStream::new(stream)),
                    _ => unreachable!(),
                };

                if collect_after {
                    Operand::Value(MetaVal::Seq(adapted_stream.collect::<Result<Vec<_>, _>>()?))
                }
                else {
                    Operand::Stream(adapted_stream)
                }

                // Operand::Value(MetaVal::Nil)
            }
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
    use super::Error;

    use bigdecimal::BigDecimal;

    use crate::metadata::resolver::ops::Operand;
    use crate::metadata::resolver::streams::Stream;

    use crate::metadata::types::MetaVal;

    use crate::test_util::TestUtil;

    fn streamify<'a, II>(mvs: II) -> Operand<'a>
    where
        II: IntoIterator<Item = MetaVal<'a>>,
    {
        let fmvs = TestUtil::create_fixed_value_stream(mvs);
        Operand::Stream(Stream::Raw(fmvs.into()))
    }

    fn assert_empty_iterable_err(result: Result<Operand<'_>, Error>) {
        match result {
            Err(Error::EmptyIterable) => {},
            _ => panic!("expected empty iterable error"),
        };
    }

    #[test]
    fn test_process() {
        positive_cases();
        negative_cases();
        preserve_stream_cases();
    }

    fn preserve_stream_cases() {
        let inputs_and_expected = vec![
            (
                (
                    UnaryOp::Flatten,
                    streamify(vec![
                        MetaVal::Int(1),
                        MetaVal::Seq(vec![
                            MetaVal::Int(2),
                            MetaVal::Int(3),
                        ]),
                        MetaVal::Int(4),
                        MetaVal::Seq(vec![
                            MetaVal::Int(5),
                            MetaVal::Int(6),
                        ]),
                        MetaVal::Int(7),
                        MetaVal::Seq(vec![]),
                    ]),
                ),
                vec![
                    MetaVal::Int(1),
                    MetaVal::Int(2),
                    MetaVal::Int(3),
                    MetaVal::Int(4),
                    MetaVal::Int(5),
                    MetaVal::Int(6),
                    MetaVal::Int(7),
                ],
            ),
            (
                (
                    UnaryOp::Flatten,
                    streamify(vec![
                        MetaVal::Int(1),
                        MetaVal::Seq(vec![
                            MetaVal::Int(2),
                            MetaVal::Int(3),
                            MetaVal::Seq(vec![
                                MetaVal::Int(4),
                                MetaVal::Int(5),
                            ]),
                        ]),
                    ]),
                ),
                vec![
                    MetaVal::Int(1),
                    MetaVal::Int(2),
                    MetaVal::Int(3),
                    MetaVal::Seq(vec![
                        MetaVal::Int(4),
                        MetaVal::Int(5),
                    ]),
                ],
            ),
            (
                (
                    UnaryOp::Flatten,
                    streamify(vec![]),
                ),
                vec![],
            ),
            (
                (
                    UnaryOp::Dedup,
                    streamify(vec![
                        MetaVal::Int(1),
                        MetaVal::Int(1),
                        MetaVal::Int(1),
                        MetaVal::Int(2),
                        MetaVal::Int(2),
                        MetaVal::Int(3),
                        MetaVal::Int(3),
                        MetaVal::Int(3),
                        MetaVal::Int(1),
                    ]),
                ),
                vec![
                    MetaVal::Int(1),
                    MetaVal::Int(2),
                    MetaVal::Int(3),
                    MetaVal::Int(1),
                ],
            ),
            (
                (
                    UnaryOp::Dedup,
                    streamify(vec![
                        MetaVal::Int(1),
                        MetaVal::Int(2),
                        MetaVal::Int(3),
                        MetaVal::Int(4),
                        MetaVal::Int(5),
                    ]),
                ),
                vec![
                    MetaVal::Int(1),
                    MetaVal::Int(2),
                    MetaVal::Int(3),
                    MetaVal::Int(4),
                    MetaVal::Int(5),
                ],
            ),
            (
                (
                    UnaryOp::Dedup,
                    streamify(vec![
                        MetaVal::Int(1),
                        MetaVal::Int(1),
                        MetaVal::Int(1),
                        MetaVal::Int(1),
                        MetaVal::Int(1),
                    ]),
                ),
                vec![
                    MetaVal::Int(1),
                ],
            ),
            (
                (
                    UnaryOp::Dedup,
                    streamify(vec![]),
                ),
                vec![],
            ),
            (
                (
                    UnaryOp::Unique,
                    streamify(vec![
                        MetaVal::Int(1),
                        MetaVal::Int(1),
                        MetaVal::Int(1),
                        MetaVal::Int(2),
                        MetaVal::Int(2),
                        MetaVal::Int(3),
                        MetaVal::Int(3),
                        MetaVal::Int(3),
                        MetaVal::Int(1),
                    ]),
                ),
                vec![
                    MetaVal::Int(1),
                    MetaVal::Int(2),
                    MetaVal::Int(3),
                ],
            ),
            (
                (
                    UnaryOp::Unique,
                    streamify(vec![
                        MetaVal::Int(1),
                        MetaVal::Int(1),
                        MetaVal::Int(1),
                        MetaVal::Int(1),
                        MetaVal::Int(1),
                    ]),
                ),
                vec![
                    MetaVal::Int(1),
                ],
            ),
            (
                (
                    UnaryOp::Unique,
                    streamify(vec![]),
                ),
                vec![],
            ),
        ];

        for (inputs, expected) in inputs_and_expected {
            let (op, input_operand) = inputs;
            let produced_operand = op.process(input_operand).unwrap();
            let produced = match produced_operand {
                Operand::Stream(stream) => stream.map(Result::unwrap).collect::<Vec<_>>(),
                _ => { panic!("expected stream as output"); },
            };

            assert_eq!(expected, produced);
        }
    }

    fn negative_cases() {
        let empty_iter_cases = vec![
            (UnaryOp::First, streamify(vec![])),
            (UnaryOp::Last, streamify(vec![])),
            (UnaryOp::MaxIn, streamify(vec![])),
            (UnaryOp::MinIn, streamify(vec![])),
        ];

        for (op, input_operand) in empty_iter_cases {
            assert_empty_iterable_err(op.process(input_operand));
        }
    }

    fn positive_cases() {
        let inputs_and_expected = vec![
            (
                (
                    UnaryOp::Collect,
                    streamify(vec![
                        TestUtil::sample_string(),
                        TestUtil::sample_integer(),
                        TestUtil::sample_boolean(),
                        TestUtil::sample_decimal(),
                        TestUtil::sample_null(),
                    ])
                ),
                MetaVal::Seq(vec![
                    TestUtil::sample_string(),
                    TestUtil::sample_integer(),
                    TestUtil::sample_boolean(),
                    TestUtil::sample_decimal(),
                    TestUtil::sample_null(),
                ]),
            ),
            (
                (
                    UnaryOp::Collect,
                    streamify(vec![])
                ),
                MetaVal::Seq(vec![]),
            ),
            (
                (
                    UnaryOp::Rev,
                    streamify(vec![
                        TestUtil::sample_string(),
                        TestUtil::sample_integer(),
                        TestUtil::sample_boolean(),
                        TestUtil::sample_decimal(),
                        TestUtil::sample_null(),
                    ])
                ),
                MetaVal::Seq(vec![
                        TestUtil::sample_null(),
                        TestUtil::sample_decimal(),
                        TestUtil::sample_boolean(),
                        TestUtil::sample_integer(),
                        TestUtil::sample_string(),
                ]),
            ),
            (
                (
                    UnaryOp::Rev,
                    streamify(vec![])
                ),
                MetaVal::Seq(vec![]),
            ),
            (
                (
                    UnaryOp::Sort,
                    streamify(vec![
                        MetaVal::Str(String::from("ab")),
                        MetaVal::Str(String::from("ca")),
                        MetaVal::Str(String::from("ac")),
                        MetaVal::Str(String::from("aa")),
                        MetaVal::Str(String::from("bc")),
                        MetaVal::Str(String::from("ba")),
                        MetaVal::Str(String::from("cc")),
                        MetaVal::Str(String::from("bb")),
                        MetaVal::Str(String::from("cb")),
                        MetaVal::Str(String::from("aaa")),
                        MetaVal::Str(String::from("bbb")),
                        MetaVal::Str(String::from("ccc")),
                    ])
                ),
                MetaVal::Seq(vec![
                        MetaVal::Str(String::from("aa")),
                        MetaVal::Str(String::from("aaa")),
                        MetaVal::Str(String::from("ab")),
                        MetaVal::Str(String::from("ac")),
                        MetaVal::Str(String::from("ba")),
                        MetaVal::Str(String::from("bb")),
                        MetaVal::Str(String::from("bbb")),
                        MetaVal::Str(String::from("bc")),
                        MetaVal::Str(String::from("ca")),
                        MetaVal::Str(String::from("cb")),
                        MetaVal::Str(String::from("cc")),
                        MetaVal::Str(String::from("ccc")),
                ]),
            ),
            (
                (
                    UnaryOp::Sort,
                    streamify(vec![
                        MetaVal::Dec(BigDecimal::new(15.into(), 1)),
                        MetaVal::Int(1),
                        MetaVal::Int(-2),
                        MetaVal::Dec(BigDecimal::new((-15).into(), 1)),
                        MetaVal::Int(-1),
                        MetaVal::Int(0),
                        MetaVal::Dec(BigDecimal::new((-5).into(), 1)),
                        MetaVal::Int(2),
                        MetaVal::Dec(BigDecimal::new(5.into(), 1)),
                    ])
                ),
                MetaVal::Seq(vec![
                        MetaVal::Int(-2),
                        MetaVal::Dec(BigDecimal::new((-15).into(), 1)),
                        MetaVal::Int(-1),
                        MetaVal::Dec(BigDecimal::new((-5).into(), 1)),
                        MetaVal::Int(0),
                        MetaVal::Dec(BigDecimal::new(5.into(), 1)),
                        MetaVal::Int(1),
                        MetaVal::Dec(BigDecimal::new(15.into(), 1)),
                        MetaVal::Int(2),
                ]),
            ),
            (
                (
                    UnaryOp::Sort,
                    streamify(vec![])
                ),
                MetaVal::Seq(vec![]),
            ),
            (
                (
                    UnaryOp::Count,
                    streamify(vec![
                        TestUtil::sample_string(),
                        TestUtil::sample_integer(),
                        TestUtil::sample_boolean(),
                        TestUtil::sample_decimal(),
                        TestUtil::sample_null(),
                    ])
                ),
                MetaVal::Int(5),
            ),
            (
                (
                    UnaryOp::Count,
                    streamify(vec![])
                ),
                MetaVal::Int(0),
            ),
            (
                (
                    UnaryOp::First,
                    streamify(vec![
                        TestUtil::sample_string(),
                        TestUtil::sample_integer(),
                        TestUtil::sample_boolean(),
                    ])
                ),
                TestUtil::sample_string(),
            ),
            (
                (
                    UnaryOp::Last,
                    streamify(vec![
                        TestUtil::sample_string(),
                        TestUtil::sample_integer(),
                        TestUtil::sample_boolean(),
                    ])
                ),
                TestUtil::sample_boolean(),
            ),
            (
                (
                    UnaryOp::MaxIn,
                    streamify(vec![
                        MetaVal::Dec(BigDecimal::new(15.into(), 1)),
                        MetaVal::Int(1),
                        MetaVal::Dec(BigDecimal::new((-15).into(), 1)),
                        MetaVal::Int(0),
                        MetaVal::Dec(BigDecimal::new((-5).into(), 1)),
                        MetaVal::Int(2),
                        MetaVal::Dec(BigDecimal::new(5.into(), 1)),
                    ])
                ),
                MetaVal::Int(2),
            ),
            (
                (
                    UnaryOp::MaxIn,
                    streamify(vec![
                        MetaVal::Dec(BigDecimal::new(15.into(), 1)),
                        MetaVal::Int(1),
                        MetaVal::Dec(BigDecimal::new((-15).into(), 1)),
                        MetaVal::Int(0),
                        MetaVal::Dec(BigDecimal::new((-5).into(), 1)),
                        MetaVal::Dec(BigDecimal::new(5.into(), 1)),
                    ])
                ),
                MetaVal::Dec(BigDecimal::new(15.into(), 1)),
            ),
            (
                (
                    UnaryOp::MinIn,
                    streamify(vec![
                        MetaVal::Dec(BigDecimal::new(15.into(), 1)),
                        MetaVal::Int(-1),
                        MetaVal::Dec(BigDecimal::new((-15).into(), 1)),
                        MetaVal::Int(0),
                        MetaVal::Dec(BigDecimal::new((-5).into(), 1)),
                        MetaVal::Int(-2),
                        MetaVal::Dec(BigDecimal::new(5.into(), 1)),
                    ])
                ),
                MetaVal::Int(-2),
            ),
            (
                (
                    UnaryOp::MinIn,
                    streamify(vec![
                        MetaVal::Dec(BigDecimal::new(15.into(), 1)),
                        MetaVal::Int(-1),
                        MetaVal::Dec(BigDecimal::new((-15).into(), 1)),
                        MetaVal::Int(0),
                        MetaVal::Dec(BigDecimal::new((-5).into(), 1)),
                        MetaVal::Dec(BigDecimal::new(5.into(), 1)),
                    ])
                ),
                MetaVal::Dec(BigDecimal::new((-15).into(), 1)),
            ),
            (
                (
                    UnaryOp::Sum,
                    streamify(vec![
                        MetaVal::Int(1),
                        MetaVal::Int(2),
                        MetaVal::Int(-3),
                    ])
                ),
                MetaVal::Int(0),
            ),
            (
                (
                    UnaryOp::Sum,
                    streamify(vec![
                        MetaVal::Int(1),
                        MetaVal::Dec(BigDecimal::new(25.into(), 1)),
                        MetaVal::Int(-3),
                    ])
                ),
                MetaVal::Dec(BigDecimal::new(5.into(), 1)),
            ),
            (
                (
                    UnaryOp::Sum,
                    streamify(vec![])
                ),
                MetaVal::Int(0),
            ),
            (
                (
                    UnaryOp::Product,
                    streamify(vec![
                        MetaVal::Int(1),
                        MetaVal::Int(2),
                        MetaVal::Int(-3),
                    ])
                ),
                MetaVal::Int(-6),
            ),
            (
                (
                    UnaryOp::Product,
                    streamify(vec![
                        MetaVal::Int(1),
                        MetaVal::Dec(BigDecimal::new(25.into(), 1)),
                        MetaVal::Int(-3),
                    ])
                ),
                MetaVal::Dec(BigDecimal::new((-75).into(), 1)),
            ),
            (
                (
                    UnaryOp::Product,
                    streamify(vec![])
                ),
                MetaVal::Int(1),
            ),
            (
                (
                    UnaryOp::AllEqual,
                    streamify(vec![
                        MetaVal::Str(String::from("same")),
                        MetaVal::Str(String::from("same")),
                        MetaVal::Str(String::from("same")),
                    ])
                ),
                MetaVal::Bul(true),
            ),
            (
                (
                    UnaryOp::AllEqual,
                    streamify(vec![
                        MetaVal::Str(String::from("same")),
                        MetaVal::Str(String::from("different")),
                        MetaVal::Str(String::from("same")),
                    ])
                ),
                MetaVal::Bul(false),
            ),
            (
                (
                    UnaryOp::AllEqual,
                    streamify(vec![
                        MetaVal::Int(1),
                        MetaVal::Int(1),
                        MetaVal::Int(1),
                    ])
                ),
                MetaVal::Bul(true),
            ),
            (
                (
                    UnaryOp::AllEqual,
                    streamify(vec![
                        MetaVal::Int(0),
                        MetaVal::Int(1),
                        MetaVal::Int(1),
                    ])
                ),
                MetaVal::Bul(false),
            ),
            // (
            //     (
            //         UnaryOp::AllEqual,
            //         streamify(vec![
            //             MetaVal::Int(1),
            //             MetaVal::Int(1),
            //             MetaVal::Dec(1.into()),
            //         ])
            //     ),
            //     MetaVal::Bul(true),
            // ),
            (
                (
                    UnaryOp::AllEqual,
                    streamify(vec![])
                ),
                MetaVal::Bul(true),
            ),

            // The following ops need to work on sequences in order to fit in this loop.
            (
                (
                    UnaryOp::Flatten,
                    Operand::Value(
                        MetaVal::Seq(vec![
                            MetaVal::Int(1),
                            MetaVal::Seq(vec![
                                MetaVal::Int(2),
                                MetaVal::Int(3),
                            ]),
                            MetaVal::Int(4),
                            MetaVal::Seq(vec![
                                MetaVal::Int(5),
                                MetaVal::Int(6),
                            ]),
                            MetaVal::Int(7),
                            MetaVal::Seq(vec![]),
                        ])
                    )
                ),
                MetaVal::Seq(vec![
                    MetaVal::Int(1),
                    MetaVal::Int(2),
                    MetaVal::Int(3),
                    MetaVal::Int(4),
                    MetaVal::Int(5),
                    MetaVal::Int(6),
                    MetaVal::Int(7),
                ]),
            ),
            (
                (
                    UnaryOp::Flatten,
                    Operand::Value(
                        MetaVal::Seq(vec![
                            MetaVal::Int(1),
                            MetaVal::Seq(vec![
                                MetaVal::Int(2),
                                MetaVal::Int(3),
                                MetaVal::Seq(vec![
                                    MetaVal::Int(4),
                                    MetaVal::Int(5),
                                ]),
                            ]),
                        ])
                    )
                ),
                MetaVal::Seq(vec![
                    MetaVal::Int(1),
                    MetaVal::Int(2),
                    MetaVal::Int(3),
                    MetaVal::Seq(vec![
                        MetaVal::Int(4),
                        MetaVal::Int(5),
                    ]),
                ]),
            ),
            (
                (
                    UnaryOp::Flatten,
                    Operand::Value(
                        MetaVal::Seq(vec![])
                    )
                ),
                MetaVal::Seq(vec![]),
            ),
            (
                (
                    UnaryOp::Dedup,
                    Operand::Value(
                        MetaVal::Seq(vec![
                            MetaVal::Int(1),
                            MetaVal::Int(1),
                            MetaVal::Int(1),
                            MetaVal::Int(2),
                            MetaVal::Int(2),
                            MetaVal::Int(3),
                            MetaVal::Int(3),
                            MetaVal::Int(3),
                            MetaVal::Int(1),
                        ])
                    )
                ),
                MetaVal::Seq(vec![
                    MetaVal::Int(1),
                    MetaVal::Int(2),
                    MetaVal::Int(3),
                    MetaVal::Int(1),
                ]),
            ),
            (
                (
                    UnaryOp::Dedup,
                    Operand::Value(
                        MetaVal::Seq(vec![
                            MetaVal::Int(1),
                            MetaVal::Int(2),
                            MetaVal::Int(3),
                            MetaVal::Int(4),
                            MetaVal::Int(5),
                        ])
                    )
                ),
                MetaVal::Seq(vec![
                    MetaVal::Int(1),
                    MetaVal::Int(2),
                    MetaVal::Int(3),
                    MetaVal::Int(4),
                    MetaVal::Int(5),
                ]),
            ),
            (
                (
                    UnaryOp::Dedup,
                    Operand::Value(
                        MetaVal::Seq(vec![
                            MetaVal::Int(1),
                            MetaVal::Int(1),
                            MetaVal::Int(1),
                            MetaVal::Int(1),
                            MetaVal::Int(1),
                        ])
                    )
                ),
                MetaVal::Seq(vec![
                    MetaVal::Int(1),
                ]),
            ),
            (
                (
                    UnaryOp::Dedup,
                    Operand::Value(
                        MetaVal::Seq(vec![])
                    )
                ),
                MetaVal::Seq(vec![]),
            ),
            (
                (
                    UnaryOp::Unique,
                    Operand::Value(
                        MetaVal::Seq(vec![
                            MetaVal::Int(1),
                            MetaVal::Int(1),
                            MetaVal::Int(1),
                            MetaVal::Int(2),
                            MetaVal::Int(2),
                            MetaVal::Int(3),
                            MetaVal::Int(3),
                            MetaVal::Int(3),
                            MetaVal::Int(1),
                        ])
                    )
                ),
                MetaVal::Seq(vec![
                    MetaVal::Int(1),
                    MetaVal::Int(2),
                    MetaVal::Int(3),
                ]),
            ),
            (
                (
                    UnaryOp::Unique,
                    Operand::Value(
                        MetaVal::Seq(vec![
                            MetaVal::Int(1),
                            MetaVal::Int(1),
                            MetaVal::Int(1),
                            MetaVal::Int(1),
                            MetaVal::Int(1),
                        ])
                    )
                ),
                MetaVal::Seq(vec![
                    MetaVal::Int(1),
                ]),
            ),
            (
                (
                    UnaryOp::Unique,
                    Operand::Value(
                        MetaVal::Seq(vec![])
                    )
                ),
                MetaVal::Seq(vec![]),
            ),
        ];

        for (inputs, expected) in inputs_and_expected {
            let (op, input_operand) = inputs;
            let produced_operand = op.process(input_operand).unwrap();
            let produced = match produced_operand {
                Operand::Value(mv) => mv,
                _ => { panic!("expected operand as output"); },
            };
            assert_eq!(expected, produced);
        }
    }
}
