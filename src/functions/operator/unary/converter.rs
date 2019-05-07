use std::convert::TryInto;
use std::cmp::Ordering;

use itertools::Itertools;
use bigdecimal::BigDecimal;

use crate::metadata::types::MetaVal;
use crate::functions::Error;
use crate::functions::operator::unary::Predicate;
use crate::functions::operator::unary::IterConsumer;

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
pub enum Converter {
    Count,
    First,
    Last,
    MaxIn,
    MinIn,
    Rev,
    Sort,
    Sum,
    Prod,
    Flatten,
    Dedup,
    Unique,
    Predicate(Predicate),
}

impl Converter {
    pub fn process<'mv>(&self, mv: MetaVal<'mv>) -> Result<MetaVal<'mv>, Error> {
        match self {
            &Self::Count => {
                let ref_seq: &Vec<_> = (&mv).try_into()?;
                Ok(MetaVal::Int(ref_seq.len() as i64))
            },
            &Self::First => {
                let seq: Vec<_> = mv.try_into()?;
                seq.into_iter().next().ok_or(Error::EmptySequence)
            },
            &Self::Last => {
                let seq: Vec<_> = mv.try_into()?;
                seq.into_iter().last().ok_or(Error::EmptySequence)
            },
            &Self::MaxIn => {
                let seq: Vec<_> = mv.try_into()?;
                IterConsumer::MaxIn.process(seq.into_iter().map(Result::Ok))
            },
            &Self::MinIn => {
                let seq: Vec<_> = mv.try_into()?;
                IterConsumer::MinIn.process(seq.into_iter().map(Result::Ok))
            }
            &Self::Rev => {
                let mut seq: Vec<_> = mv.try_into()?;
                seq.reverse();
                Ok(MetaVal::Seq(seq))
            },
            &Self::Sort => {
                let mut seq: Vec<_> = mv.try_into()?;
                seq.sort_by(smart_sort_by);
                Ok(MetaVal::Seq(seq))
            },
            &Self::Sum => {
                let seq: Vec<_> = mv.try_into()?;
                IterConsumer::Sum.process(seq.into_iter().map(Result::Ok))
            },
            &Self::Prod => {
                let seq: Vec<_> = mv.try_into()?;
                IterConsumer::Prod.process(seq.into_iter().map(Result::Ok))
            },
            &Self::Flatten => {
                let seq: Vec<_> = mv.try_into()?;
                let mut flattened = vec![];

                for mv in seq {
                    match mv {
                        MetaVal::Seq(mut seq) => flattened.append(&mut seq),
                        mv => flattened.push(mv),
                    }
                }
                Ok(MetaVal::Seq(flattened))
            },
            &Self::Dedup => {
                let mut seq: Vec<_> = mv.try_into()?;
                // TODO: Figure out equality rules.
                seq.dedup();
                Ok(MetaVal::Seq(seq))
            },
            &Self::Unique => {
                let seq: Vec<_> = mv.try_into()?;
                // TODO: Figure out equality rules.
                Ok(MetaVal::Seq(seq.into_iter().unique().collect()))
            },

            // All predicates are implicitly converters as well.
            &Self::Predicate(pred) => pred.process(&mv).map(MetaVal::Bul),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Converter;

    use crate::test_util::TestUtil as TU;

    use bigdecimal::BigDecimal;

    use crate::metadata::types::MetaVal;
    use crate::functions::Error;

    fn i(i: i64) -> MetaVal<'static> {
        MetaVal::Int(i)
    }

    fn d(i: i64, e: i64) -> MetaVal<'static> {
        MetaVal::Dec(BigDecimal::new(i.into(), e))
    }

    fn positive_cases() {
        let inputs_and_expected = vec![
            (
                (Converter::Count, TU::sample_flat_sequence()),
                MetaVal::Int(5),
            ),
            (
                (Converter::Count, MetaVal::Seq(vec![])),
                MetaVal::Int(0),
            ),
            (
                (Converter::First, TU::sample_flat_sequence()),
                TU::sample_string(),
            ),
            (
                (Converter::First, MetaVal::Seq(vec![TU::sample_string()])),
                TU::sample_string(),
            ),
            (
                (Converter::Last, TU::sample_flat_sequence()),
                TU::sample_null(),
            ),
            (
                (Converter::Last, MetaVal::Seq(vec![TU::sample_string()])),
                TU::sample_string(),
            ),
            (
                (Converter::MaxIn, TU::sample_number_sequence(2, false, true, true)),
                MetaVal::Int(2),
            ),
            (
                (Converter::MaxIn, TU::sample_number_sequence(2, true, true, true)),
                MetaVal::Dec(2.5.into()),
            ),
            (
                (Converter::MinIn, TU::sample_number_sequence(2, false, true, true)),
                MetaVal::Int(-2),
            ),
            (
                (Converter::MinIn, TU::sample_number_sequence(2, true, true, true)),
                MetaVal::Dec((-2.5).into()),
            ),
            (
                (Converter::Rev, MetaVal::Seq(vec![TU::sample_boolean(), TU::sample_decimal(), TU::sample_integer()])),
                MetaVal::Seq(vec![TU::sample_integer(), TU::sample_decimal(), TU::sample_boolean()]),
            ),
            (
                (Converter::Rev, MetaVal::Seq(vec![])),
                MetaVal::Seq(vec![]),
            ),
            (
                (Converter::Sort, TU::sample_flat_sequence()),
                MetaVal::Seq(vec![TU::sample_null(), TU::sample_string(), TU::sample_integer(), TU::sample_boolean(), TU::sample_decimal()]),
            ),
            (
                (Converter::Sort, TU::sample_number_sequence(1, true, true, true)),
                MetaVal::Seq(vec![d(-15, 1), i(-1), d(-5, 1), i(0), d(5, 1), i(1), d(15, 1)]),
            ),
            (
                (Converter::Sort, MetaVal::Seq(vec![])),
                MetaVal::Seq(vec![]),
            ),
            (
                (Converter::Sum, MetaVal::Seq(vec![i(-2), i(3), i(5), i(7)])),
                i(-2 + 3 + 5 + 7),
            ),
            (
                (Converter::Sum, MetaVal::Seq(vec![i(-2), i(3), d(55, 1), i(7)])),
                d(135, 1),
            ),
            (
                (Converter::Sum, MetaVal::Seq(vec![])),
                i(0),
            ),
            (
                (Converter::Prod, MetaVal::Seq(vec![i(-2), i(3), i(5), i(7)])),
                i(-2 * 3 * 5 * 7),
            ),
            (
                (Converter::Prod, MetaVal::Seq(vec![i(-2), i(3), d(55, 1), i(7)])),
                d(-231, 0),
            ),
            (
                (Converter::Prod, MetaVal::Seq(vec![i(0), i(-2), i(3), d(55, 1), i(7)])),
                d(0, 0),
            ),
            (
                (Converter::Prod, MetaVal::Seq(vec![])),
                i(1),
            ),
        ];

        for (inputs, expected) in inputs_and_expected {
            let (conv, mv) = inputs;
            let produced = conv.process(mv).unwrap();
            assert_eq!(expected, produced);
        }
    }

    fn negative_cases() {
    }

    #[test]
    fn test_process() {
        positive_cases();
        negative_cases();
    }
}
