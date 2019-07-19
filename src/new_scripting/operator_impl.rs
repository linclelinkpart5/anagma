use std::borrow::Cow;

use crate::metadata::types::MetaVal;
use crate::new_scripting::Error;

pub fn collect<'a>(it: impl Iterator<Item = Result<MetaVal, Error>>) -> Result<Vec<MetaVal>, Error> {
    let mut ret = vec![];
    for res in it { ret.push(res?); }
    Ok(ret)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::new_scripting::ErrorKind;

    use crate::test_util::TestUtil as TU;

    enum Prod {
        Fixed(Vec<MetaVal>),
        Raw(Vec<Result<MetaVal, Error>>),
    }

    impl IntoIterator for Prod {
        type Item = Result<MetaVal, Error>;
        type IntoIter = Iter;

        fn into_iter(self) -> Self::IntoIter {
            match self {
                Prod::Fixed(v) => Iter::Fixed(v.into_iter()),
                Prod::Raw(v) => Iter::Raw(v.into_iter()),
            }
        }
    }

    enum Iter {
        Fixed(std::vec::IntoIter<MetaVal>),
        Raw(std::vec::IntoIter<Result<MetaVal, Error>>),
    }

    impl Iterator for Iter {
        type Item = Result<MetaVal, Error>;

        fn next(&mut self) -> Option<Self::Item> {
            match self {
                Iter::Fixed(ref mut it) => it.next().map(Result::Ok),
                Iter::Raw(ref mut it) => it.next(),
            }
        }
    }

    #[test]
    fn test_collect() {
        let inputs_and_expected = vec![
            (
                Prod::Fixed(vec![]),
                Ok(vec![]),
            ),
            (
                Prod::Fixed(TU::core_nested_sequence()),
                Ok(TU::core_nested_sequence()),
            ),
            (
                Prod::Raw(vec![Err(Error::Sentinel)]),
                Err(ErrorKind::Sentinel),
            ),
            (
                Prod::Raw(vec![Ok(TU::b(true)), Ok(TU::b(true)), Err(Error::Sentinel)]),
                Err(ErrorKind::Sentinel),
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let produced = collect(input.into_iter()).map_err(ErrorKind::from);
            assert_eq!(expected, produced);
        }
    }
}
