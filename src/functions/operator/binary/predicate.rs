use std::convert::TryInto;

use crate::metadata::types::MetaVal;
use crate::functions::Error;

#[derive(Clone, Copy, Debug)]
pub enum Predicate {
    AllEqual,
}

impl Predicate {
    pub fn process<'mv>(&self, mv: &'mv MetaVal<'mv>) -> Result<bool, Error> {
        match self {
            &Self::AllEqual => {
                let ref_seq: &Vec<_> = mv.try_into()?;

                let mut it = ref_seq.into_iter();

                match it.next() {
                    None => Ok(true),
                    Some(first_mv) => {
                        for mv in it {
                            if mv != first_mv { return Ok(false); }
                        }

                        Ok(true)
                    },
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Predicate;

    use crate::metadata::types::MetaVal;
    use crate::functions::Error;

    fn positive_cases() {
        let inputs_and_expected = vec![
            (
                (
                    Predicate::AllEqual,
                    MetaVal::Seq(vec![
                        MetaVal::Int(1),
                        MetaVal::Int(2),
                        MetaVal::Int(3),
                    ]),
                ),
                false,
            ),
            (
                (
                    Predicate::AllEqual,
                    MetaVal::Seq(vec![
                        MetaVal::Int(1),
                        MetaVal::Int(1),
                        MetaVal::Int(1),
                    ]),
                ),
                true,
            ),
            (
                (
                    Predicate::AllEqual,
                    MetaVal::Seq(vec![]),
                ),
                true,
            ),
        ];

        for (inputs, expected) in inputs_and_expected {
            let (pred, mv) = inputs;
            let produced = pred.process(&mv).unwrap();
            assert_eq!(expected, produced);
        }
    }

    fn negative_cases() {
        let not_sequence_err_cases = vec![
            (Predicate::AllEqual, MetaVal::Nil),
        ];

        for (pred, mv) in not_sequence_err_cases {
            match pred.process(&mv) {
                Err(Error::NotSequence) => {},
                _ => panic!("expected a failure case"),
            }
        }
    }

    #[test]
    fn test_process() {
        positive_cases();
        negative_cases();
    }
}
