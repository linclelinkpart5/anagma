pub mod converter;
pub mod predicate;
pub mod iter_consumer;
pub mod iter_adaptor;

pub use self::converter::Converter;
pub use self::predicate::Predicate;
pub use self::iter_consumer::IterConsumer;
pub use self::iter_adaptor::IterAdaptor;

use crate::metadata::types::MetaVal;
use crate::functions::Error;
use crate::functions::util::value_producer::ValueProducer;
use crate::functions::util::value_producer::Fixed;
use crate::functions::util::value_producer::Filter;
use crate::functions::util::value_producer::Map;
use crate::functions::util::value_producer::StepBy;
use crate::functions::util::value_producer::Chain;
use crate::functions::util::value_producer::Zip;
use crate::functions::util::value_producer::Skip;
use crate::functions::util::value_producer::Take;
use crate::functions::util::value_producer::SkipWhile;
use crate::functions::util::value_producer::TakeWhile;
use crate::functions::util::value_producer::Intersperse;
use crate::functions::util::value_producer::Interleave;
use crate::functions::util::UnaryPred;
use crate::functions::util::UnaryConv;

#[derive(Clone, Copy)]
enum AllAny { All, Any, }

impl AllAny {
    fn target(self) -> bool {
        match self {
            Self::All => false,
            Self::Any => true,
        }
    }
}

/// Namespace for all the implementation of various functions in this module.
pub struct Impl;

impl Impl {
    pub fn nth<'a, VP: ValueProducer<'a>>(vp: VP, n: usize) -> Result<MetaVal<'a>, Error> {
        let mut i = 0;
        for res_mv in vp {
            let mv = res_mv?;

            if i == n { return Ok(mv) }
            else { i += 1; }
        }

        Err(Error::OutOfBounds)
    }

    pub fn nth_s(seq: Vec<MetaVal>, n: usize) -> Result<MetaVal, Error> {
        seq.into_iter().nth(n).ok_or(Error::OutOfBounds)
    }

    fn all_any<'a, VP: ValueProducer<'a>>(vp: VP, u_pred: UnaryPred, flag: AllAny) -> Result<bool, Error> {
        let target = flag.target();
        for res_mv in vp {
            let mv = res_mv?;
            if u_pred(&mv)? == target { return Ok(target) }
        }

        Ok(!target)
    }

    pub fn all<'a, VP: ValueProducer<'a>>(vp: VP, u_pred: UnaryPred) -> Result<bool, Error> {
        Self::all_any(vp, u_pred, AllAny::All)
    }

    pub fn all_s(seq: Vec<MetaVal>, u_pred: UnaryPred) -> Result<bool, Error> {
        Self::all_any(Fixed::new(seq), u_pred, AllAny::All)
    }

    pub fn any<'a, VP: ValueProducer<'a>>(vp: VP, u_pred: UnaryPred) -> Result<bool, Error> {
        Self::all_any(vp, u_pred, AllAny::Any)
    }

    pub fn any_s(seq: Vec<MetaVal>, u_pred: UnaryPred) -> Result<bool, Error> {
        Self::all_any(Fixed::new(seq), u_pred, AllAny::Any)
    }

    pub fn find<'a, VP: ValueProducer<'a>>(vp: VP, u_pred: UnaryPred) -> Result<MetaVal<'a>, Error> {
        for res_mv in vp {
            let mv = res_mv?;
            if u_pred(&mv)? { return Ok(mv) }
        }

        Err(Error::ItemNotFound)
    }

    pub fn find_s(seq: Vec<MetaVal>, u_pred: UnaryPred) -> Result<MetaVal, Error> {
        Self::find(Fixed::new(seq), u_pred)
    }

    pub fn position<'a, VP: ValueProducer<'a>>(vp: VP, u_pred: UnaryPred) -> Result<usize, Error> {
        let mut i = 0;
        for res_mv in vp {
            let mv = res_mv?;
            if u_pred(&mv)? { return Ok(i) }
            i += 1;
        }

        Err(Error::ItemNotFound)
    }

    pub fn position_s(seq: Vec<MetaVal>, u_pred: UnaryPred) -> Result<usize, Error> {
        Self::position(Fixed::new(seq), u_pred)
    }

    pub fn filter<'a, VP: ValueProducer<'a>>(vp: VP, u_pred: UnaryPred) -> Filter<VP> {
        Filter::new(vp, u_pred)
    }

    pub fn filter_s(seq: Vec<MetaVal>, u_pred: UnaryPred) -> Result<Vec<MetaVal>, Error> {
        // It is possible for the predicate to fail.
        Filter::new(Fixed::new(seq), u_pred).collect()
    }

    pub fn map<'a, VP: ValueProducer<'a>>(vp: VP, u_conv: UnaryConv) -> Map<VP> {
        Map::new(vp, u_conv)
    }

    pub fn map_s(seq: Vec<MetaVal>, u_conv: UnaryConv) -> Result<Vec<MetaVal>, Error> {
        // It is possible for the converter to fail.
        Map::new(Fixed::new(seq), u_conv).collect()
    }

    pub fn step_by<'a, VP: ValueProducer<'a>>(vp: VP, step: usize) -> Result<StepBy<VP>, Error> {
        StepBy::new(vp, step)
    }

    pub fn step_by_s(seq: Vec<MetaVal>, step: usize) -> Result<Vec<MetaVal>, Error> {
        // It is possible for the step by producer creation to fail.
        // NOTE: The match is not needed, but it seems desirable to make explicit that the collect cannot fail.
        match StepBy::new(Fixed::new(seq), step)?.collect::<Result<Vec<MetaVal>, _>>() {
            Err(_) => unreachable!(),
            Ok(seq) => Ok(seq),
        }
    }

    pub fn chain<'a, VPA: ValueProducer<'a>, VPB: ValueProducer<'a>>(vp_a: VPA, vp_b: VPB) -> Chain<VPA, VPB> {
        Chain::new(vp_a, vp_b)
    }

    pub fn chain_s<'a>(seq_a: Vec<MetaVal<'a>>, seq_b: Vec<MetaVal<'a>>) -> Vec<MetaVal<'a>> {
        let mut seq_a = seq_a;
        seq_a.extend(seq_b);
        seq_a
    }

    pub fn zip<'a, VPA: ValueProducer<'a>, VPB: ValueProducer<'a>>(vp_a: VPA, vp_b: VPB) -> Zip<VPA, VPB> {
        Zip::new(vp_a, vp_b)
    }

    pub fn zip_s<'a>(seq_a: Vec<MetaVal<'a>>, seq_b: Vec<MetaVal<'a>>) -> Vec<MetaVal<'a>> {
        // Zipping cannot fail.
        match Zip::new(Fixed::new(seq_a), Fixed::new(seq_b)).collect::<Result<Vec<MetaVal>, _>>() {
            Err(_) => unreachable!(),
            Ok(seq) => seq,
        }
    }

    pub fn skip<'a, VP: ValueProducer<'a>>(vp: VP, n: usize) -> Skip<'a, VP> {
        Skip::new(vp, n)
    }

    pub fn skip_s(seq: Vec<MetaVal>, n: usize) -> Vec<MetaVal> {
        seq.into_iter().skip(n).collect()
    }

    pub fn take<'a, VP: ValueProducer<'a>>(vp: VP, n: usize) -> Take<'a, VP> {
        Take::new(vp, n)
    }

    pub fn take_s(seq: Vec<MetaVal>, n: usize) -> Vec<MetaVal> {
        seq.into_iter().take(n).collect()
    }

    pub fn skip_while<'a, VP: ValueProducer<'a>>(vp: VP, u_pred: UnaryPred) -> SkipWhile<VP> {
        SkipWhile::new(vp, u_pred)
    }

    pub fn skip_while_s(seq: Vec<MetaVal>, u_pred: UnaryPred) -> Result<Vec<MetaVal>, Error> {
        // It is possible for the predicate to fail.
        SkipWhile::new(Fixed::new(seq), u_pred).collect()
    }

    pub fn take_while<'a, VP: ValueProducer<'a>>(vp: VP, u_pred: UnaryPred) -> TakeWhile<VP> {
        TakeWhile::new(vp, u_pred)
    }

    pub fn take_while_s(seq: Vec<MetaVal>, u_pred: UnaryPred) -> Result<Vec<MetaVal>, Error> {
        // It is possible for the predicate to fail.
        TakeWhile::new(Fixed::new(seq), u_pred).collect()
    }

    pub fn intersperse<'a, VP: ValueProducer<'a>>(vp: VP, mv: MetaVal<'a>) -> Intersperse<'a, VP> {
        Intersperse::new(vp, mv)
    }

    pub fn intersperse_s<'a>(seq: Vec<MetaVal<'a>>, mv: MetaVal<'a>) -> Vec<MetaVal<'a>> {
        // Interspersing cannot fail.
        match Intersperse::new(Fixed::new(seq), mv).collect::<Result<Vec<MetaVal>, _>>() {
            Err(_) => unreachable!(),
            Ok(seq) => seq,
        }
    }

    pub fn interleave<'a, VPA: ValueProducer<'a>, VPB: ValueProducer<'a>>(vp_a: VPA, vp_b: VPB) -> Interleave<VPA, VPB> {
        Interleave::new(vp_a, vp_b)
    }

    pub fn interleave_s<'a>(seq_a: Vec<MetaVal<'a>>, seq_b: Vec<MetaVal<'a>>) -> Vec<MetaVal<'a>> {
        // Interleaving cannot fail.
        match Interleave::new(Fixed::new(seq_a), Fixed::new(seq_b)).collect::<Result<Vec<MetaVal>, _>>() {
            Err(_) => unreachable!(),
            Ok(seq) => seq,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Impl;

    use crate::test_util::TestUtil as TU;

    use crate::metadata::types::MetaVal;
    use crate::functions::Error;
    use crate::functions::ErrorKind;
    use crate::functions::util::value_producer::Raw;
    use crate::functions::util::NumberLike;

    fn is_even_int(mv: &MetaVal) -> Result<bool, Error> {
        match mv {
            MetaVal::Int(i) => Ok(i % 2 == 0),
            _ => Err(Error::NotNumeric),
        }
    }

    fn is_boolean(mv: &MetaVal) -> Result<bool, Error> {
        match mv {
            MetaVal::Bul(..) => Ok(true),
            _ => Ok(false),
        }
    }

    fn is_integer(mv: &MetaVal) -> Result<bool, Error> {
        match mv {
            MetaVal::Int(..) => Ok(true),
            _ => Ok(false),
        }
    }

    fn is_gt_6_int(mv: &MetaVal) -> Result<bool, Error> {
        match mv {
            MetaVal::Int(i) => Ok(i > &6),
            _ => Err(Error::NotNumeric),
        }
    }

    fn conv_repr(mv: MetaVal) -> Result<MetaVal, Error> {
        Ok(
            MetaVal::Str(
                match mv {
                    MetaVal::Bul(..) => "boolean",
                    MetaVal::Dec(..) => "decimal",
                    MetaVal::Int(..) => "integer",
                    MetaVal::Map(..) => "mapping",
                    MetaVal::Nil => "null",
                    MetaVal::Seq(..) => "sequence",
                    MetaVal::Str(..) => "string",
                }.to_string()
            )
        )
    }

    fn conv_add_3(mv: MetaVal) -> Result<MetaVal, Error> {
        match mv {
            MetaVal::Dec(d) => Ok(MetaVal::Dec(d + dec!(3))),
            MetaVal::Int(i) => Ok(MetaVal::Int(i + 3)),
            _ => Err(Error::NotNumeric),
        }
    }

    #[test]
    fn test_nth() {
        let inputs_and_expected = vec![
            (
                (vec![], 1usize),
                Err(ErrorKind::OutOfBounds),
            ),
            (
                (TU::core_nested_sequence().into_iter().map(Result::Ok).collect(), 0),
                Ok(TU::sample_string()),
            ),
            (
                (TU::core_nested_sequence().into_iter().map(Result::Ok).collect(), 100),
                Err(ErrorKind::OutOfBounds),
            ),
            (
                (vec![Ok(MetaVal::Bul(true)), Ok(MetaVal::Bul(true)), Err(Error::Sentinel)], 1),
                Ok(MetaVal::Bul(true)),
            ),
            (
                (vec![Err(Error::Sentinel), Ok(MetaVal::Bul(true)), Ok(MetaVal::Bul(true))], 1),
                Err(ErrorKind::Sentinel),
            ),
        ];

        for (inputs, expected) in inputs_and_expected {
            let (input_a, input_b) = inputs;
            let produced = Impl::nth(Raw::new(input_a), input_b).map_err(Into::<ErrorKind>::into);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_nth_s() {
        let inputs_and_expected = vec![
            (
                (vec![], 1usize),
                Err(ErrorKind::OutOfBounds),
            ),
            (
                (TU::core_nested_sequence(), 0),
                Ok(TU::sample_string()),
            ),
            (
                (TU::core_nested_sequence(), 100),
                Err(ErrorKind::OutOfBounds),
            ),
            (
                (vec![MetaVal::Bul(true), MetaVal::Bul(true)], 1),
                Ok(MetaVal::Bul(true)),
            ),
        ];

        for (inputs, expected) in inputs_and_expected {
            let (input_a, input_b) = inputs;
            let produced = Impl::nth_s(input_a, input_b).map_err(Into::<ErrorKind>::into);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_all() {
        let inputs_and_expected: Vec<((_, fn(&MetaVal) -> Result<bool, Error>), _)> = vec![
            (
                (vec![], is_boolean),
                Ok(true),
            ),
            (
                (TU::core_nested_sequence().into_iter().map(Result::Ok).collect(), is_boolean),
                Ok(false),
            ),
            (
                (vec![Ok(MetaVal::Bul(true)), Ok(MetaVal::Bul(true)), Err(Error::Sentinel)], is_boolean),
                Err(ErrorKind::Sentinel),
            ),
            (
                (vec![Err(Error::Sentinel), Ok(MetaVal::Bul(true)), Ok(MetaVal::Bul(true))], is_boolean),
                Err(ErrorKind::Sentinel),
            ),
            (
                (vec![Ok(MetaVal::Bul(true)), Ok(MetaVal::Int(0)), Err(Error::Sentinel)], is_boolean),
                Ok(false),
            ),
            (
                (vec![Ok(TU::i(0)), Ok(TU::i(2)), Ok(TU::i(4)), Ok(TU::i(6)), Ok(TU::i(8))], is_even_int),
                Ok(true),
            ),
            (
                (vec![Ok(TU::i(0)), Ok(TU::i(2)), Ok(TU::i(5)), Ok(TU::i(6)), Ok(TU::i(8))], is_even_int),
                Ok(false),
            ),
            (
                (vec![Ok(TU::i(1)), Ok(TU::i(3)), Ok(TU::i(5)), Ok(TU::i(7)), Ok(TU::i(9))], is_even_int),
                Ok(false),
            ),
            (
                (vec![Ok(TU::i(0)), Ok(TU::i(2)), Ok(MetaVal::Bul(false)), Ok(TU::i(6)), Ok(TU::i(8))], is_even_int),
                Err(ErrorKind::NotNumeric),
            ),
            (
                (vec![Ok(TU::i(1)), Ok(TU::i(3)), Ok(MetaVal::Bul(false)), Ok(TU::i(7)), Ok(TU::i(9))], is_even_int),
                Ok(false),
            ),
        ];

        for (inputs, expected) in inputs_and_expected {
            let (input_a, input_b) = inputs;
            let produced = Impl::all(Raw::new(input_a), input_b).map_err(Into::<ErrorKind>::into);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_all_s() {
        let inputs_and_expected: Vec<((_, fn(&MetaVal) -> Result<bool, Error>), _)> = vec![
            (
                (vec![], is_boolean),
                Ok(true),
            ),
            (
                (TU::core_nested_sequence(), is_boolean),
                Ok(false),
            ),
            (
                (vec![MetaVal::Bul(true), MetaVal::Bul(true)], is_boolean),
                Ok(true),
            ),
            (
                (vec![MetaVal::Bul(true), MetaVal::Int(0)], is_boolean),
                Ok(false),
            ),
            (
                (vec![TU::i(0), TU::i(2), TU::i(4), TU::i(6), TU::i(8)], is_even_int),
                Ok(true),
            ),
            (
                (vec![TU::i(0), TU::i(2), TU::i(5), TU::i(6), TU::i(8)], is_even_int),
                Ok(false),
            ),
            (
                (vec![TU::i(1), TU::i(3), TU::i(5), TU::i(7), TU::i(9)], is_even_int),
                Ok(false),
            ),
            (
                (vec![TU::i(0), TU::i(2), MetaVal::Bul(false), TU::i(6), TU::i(8)], is_even_int),
                Err(ErrorKind::NotNumeric),
            ),
            (
                (vec![TU::i(1), TU::i(3), MetaVal::Bul(false), TU::i(7), TU::i(9)], is_even_int),
                Ok(false),
            ),
        ];

        for (inputs, expected) in inputs_and_expected {
            let (input_a, input_b) = inputs;
            let produced = Impl::all_s(input_a, input_b).map_err(Into::<ErrorKind>::into);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_any() {
        let inputs_and_expected: Vec<((_, fn(&MetaVal) -> Result<bool, Error>), _)> = vec![
            (
                (vec![], is_boolean),
                Ok(false),
            ),
            (
                (TU::core_nested_sequence().into_iter().map(Result::Ok).collect(), is_boolean),
                Ok(true),
            ),
            (
                (vec![Ok(MetaVal::Bul(true)), Ok(MetaVal::Bul(true)), Err(Error::Sentinel)], is_boolean),
                Ok(true),
            ),
            (
                (vec![Err(Error::Sentinel), Ok(MetaVal::Bul(true)), Ok(MetaVal::Bul(true))], is_boolean),
                Err(ErrorKind::Sentinel),
            ),
            (
                (vec![Ok(MetaVal::Bul(true)), Ok(MetaVal::Int(0)), Err(Error::Sentinel)], is_boolean),
                Ok(true),
            ),
            (
                (vec![Ok(TU::i(0)), Ok(TU::i(2)), Ok(TU::i(4)), Ok(TU::i(6)), Ok(TU::i(8))], is_even_int),
                Ok(true),
            ),
            (
                (vec![Ok(TU::i(0)), Ok(TU::i(2)), Ok(TU::i(5)), Ok(TU::i(6)), Ok(TU::i(8))], is_even_int),
                Ok(true),
            ),
            (
                (vec![Ok(TU::i(1)), Ok(TU::i(3)), Ok(TU::i(5)), Ok(TU::i(7)), Ok(TU::i(9))], is_even_int),
                Ok(false),
            ),
            (
                (vec![Ok(TU::i(0)), Ok(TU::i(2)), Ok(MetaVal::Bul(false)), Ok(TU::i(6)), Ok(TU::i(8))], is_even_int),
                Ok(true),
            ),
            (
                (vec![Ok(TU::i(1)), Ok(TU::i(3)), Ok(MetaVal::Bul(false)), Ok(TU::i(7)), Ok(TU::i(9))], is_even_int),
                Err(ErrorKind::NotNumeric),
            ),
        ];

        for (inputs, expected) in inputs_and_expected {
            let (input_a, input_b) = inputs;
            let produced = Impl::any(Raw::new(input_a), input_b).map_err(Into::<ErrorKind>::into);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_any_s() {
        let inputs_and_expected: Vec<((_, fn(&MetaVal) -> Result<bool, Error>), _)> = vec![
            (
                (vec![], is_boolean),
                Ok(false),
            ),
            (
                (TU::core_nested_sequence(), is_boolean),
                Ok(true),
            ),
            (
                (vec![MetaVal::Bul(true), MetaVal::Bul(true)], is_boolean),
                Ok(true),
            ),
            (
                (vec![MetaVal::Bul(true), MetaVal::Int(0)], is_boolean),
                Ok(true),
            ),
            (
                (vec![TU::i(0), TU::i(2), TU::i(4), TU::i(6), TU::i(8)], is_even_int),
                Ok(true),
            ),
            (
                (vec![TU::i(0), TU::i(2), TU::i(5), TU::i(6), TU::i(8)], is_even_int),
                Ok(true),
            ),
            (
                (vec![TU::i(1), TU::i(3), TU::i(5), TU::i(7), TU::i(9)], is_even_int),
                Ok(false),
            ),
            (
                (vec![TU::i(0), TU::i(2), MetaVal::Bul(false), TU::i(6), TU::i(8)], is_even_int),
                Ok(true),
            ),
            (
                (vec![TU::i(1), TU::i(3), MetaVal::Bul(false), TU::i(7), TU::i(9)], is_even_int),
                Err(ErrorKind::NotNumeric),
            ),
        ];

        for (inputs, expected) in inputs_and_expected {
            let (input_a, input_b) = inputs;
            let produced = Impl::any_s(input_a, input_b).map_err(Into::<ErrorKind>::into);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_find() {
        let inputs_and_expected: Vec<((_, fn(&MetaVal) -> Result<bool, Error>), _)> = vec![
            (
                (vec![], is_boolean),
                Err(ErrorKind::ItemNotFound),
            ),
            (
                (TU::core_nested_sequence().into_iter().map(Result::Ok).collect(), is_boolean),
                Ok(TU::sample_boolean()),
            ),
            (
                (vec![Ok(MetaVal::Bul(false)), Ok(MetaVal::Bul(true)), Err(Error::Sentinel)], is_boolean),
                Ok(MetaVal::Bul(false)),
            ),
            (
                (vec![Err(Error::Sentinel), Ok(MetaVal::Bul(true)), Ok(MetaVal::Bul(true))], is_boolean),
                Err(ErrorKind::Sentinel),
            ),
            (
                (vec![Ok(TU::i(0)), Ok(TU::i(2)), Ok(TU::i(4)), Ok(TU::i(6)), Ok(TU::i(8))], is_even_int),
                Ok(TU::i(0)),
            ),
            (
                (vec![Ok(TU::i(0)), Ok(TU::i(2)), Ok(TU::i(5)), Ok(TU::i(6)), Ok(TU::i(8))], is_even_int),
                Ok(TU::i(0)),
            ),
            (
                (vec![Ok(TU::i(1)), Ok(TU::i(3)), Ok(TU::i(5)), Ok(TU::i(7)), Ok(TU::i(9))], is_even_int),
                Err(ErrorKind::ItemNotFound),
            ),
            (
                (vec![Ok(TU::i(0)), Ok(TU::i(2)), Ok(MetaVal::Bul(false)), Ok(TU::i(6)), Ok(TU::i(8))], is_even_int),
                Ok(TU::i(0)),
            ),
            (
                (vec![Ok(TU::i(1)), Ok(TU::i(3)), Ok(MetaVal::Bul(false)), Ok(TU::i(7)), Ok(TU::i(9))], is_even_int),
                Err(ErrorKind::NotNumeric),
            ),
        ];

        for (inputs, expected) in inputs_and_expected {
            let (input_a, input_b) = inputs;
            let produced = Impl::find(Raw::new(input_a), input_b).map_err(Into::<ErrorKind>::into);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_find_s() {
        let inputs_and_expected: Vec<((_, fn(&MetaVal) -> Result<bool, Error>), _)> = vec![
            (
                (vec![], is_boolean),
                Err(ErrorKind::ItemNotFound),
            ),
            (
                (TU::core_nested_sequence(), is_boolean),
                Ok(TU::sample_boolean()),
            ),
            (
                (vec![MetaVal::Bul(false), MetaVal::Bul(true)], is_boolean),
                Ok(MetaVal::Bul(false)),
            ),
            (
                (vec![TU::i(0), TU::i(2), TU::i(4), TU::i(6), TU::i(8)], is_even_int),
                Ok(TU::i(0)),
            ),
            (
                (vec![TU::i(0), TU::i(2), TU::i(5), TU::i(6), TU::i(8)], is_even_int),
                Ok(TU::i(0)),
            ),
            (
                (vec![TU::i(1), TU::i(3), TU::i(5), TU::i(7), TU::i(9)], is_even_int),
                Err(ErrorKind::ItemNotFound),
            ),
            (
                (vec![TU::i(0), TU::i(2), MetaVal::Bul(false), TU::i(6), TU::i(8)], is_even_int),
                Ok(TU::i(0)),
            ),
            (
                (vec![TU::i(1), TU::i(3), MetaVal::Bul(false), TU::i(7), TU::i(9)], is_even_int),
                Err(ErrorKind::NotNumeric),
            ),
        ];

        for (inputs, expected) in inputs_and_expected {
            let (input_a, input_b) = inputs;
            let produced = Impl::find_s(input_a, input_b).map_err(Into::<ErrorKind>::into);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_position() {
        let inputs_and_expected: Vec<((_, fn(&MetaVal) -> Result<bool, Error>), _)> = vec![
            (
                (vec![], is_boolean),
                Err(ErrorKind::ItemNotFound),
            ),
            (
                (TU::core_nested_sequence().into_iter().map(Result::Ok).collect(), is_boolean),
                Ok(3),
            ),
            (
                (vec![Ok(MetaVal::Bul(false)), Ok(MetaVal::Bul(true)), Err(Error::Sentinel)], is_boolean),
                Ok(0),
            ),
            (
                (vec![Err(Error::Sentinel), Ok(MetaVal::Bul(true)), Ok(MetaVal::Bul(true))], is_boolean),
                Err(ErrorKind::Sentinel),
            ),
            (
                (vec![Ok(TU::i(0)), Ok(TU::i(2)), Ok(TU::i(4)), Ok(TU::i(6)), Ok(TU::i(8))], is_even_int),
                Ok(0),
            ),
            (
                (vec![Ok(TU::i(0)), Ok(TU::i(2)), Ok(TU::i(5)), Ok(TU::i(6)), Ok(TU::i(8))], is_even_int),
                Ok(0),
            ),
            (
                (vec![Ok(TU::i(1)), Ok(TU::i(3)), Ok(TU::i(5)), Ok(TU::i(7)), Ok(TU::i(9))], is_even_int),
                Err(ErrorKind::ItemNotFound),
            ),
            (
                (vec![Ok(TU::i(0)), Ok(TU::i(2)), Ok(MetaVal::Bul(false)), Ok(TU::i(6)), Ok(TU::i(8))], is_even_int),
                Ok(0),
            ),
            (
                (vec![Ok(TU::i(1)), Ok(TU::i(3)), Ok(MetaVal::Bul(false)), Ok(TU::i(7)), Ok(TU::i(9))], is_even_int),
                Err(ErrorKind::NotNumeric),
            ),
        ];

        for (inputs, expected) in inputs_and_expected {
            let (input_a, input_b) = inputs;
            let produced = Impl::position(Raw::new(input_a), input_b).map_err(Into::<ErrorKind>::into);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_position_s() {
        let inputs_and_expected: Vec<((_, fn(&MetaVal) -> Result<bool, Error>), _)> = vec![
            (
                (vec![], is_boolean),
                Err(ErrorKind::ItemNotFound),
            ),
            (
                (TU::core_nested_sequence(), is_boolean),
                Ok(3),
            ),
            (
                (vec![MetaVal::Bul(false), MetaVal::Bul(true)], is_boolean),
                Ok(0),
            ),
            (
                (vec![TU::i(0), TU::i(2), TU::i(4), TU::i(6), TU::i(8)], is_even_int),
                Ok(0),
            ),
            (
                (vec![TU::i(0), TU::i(2), TU::i(5), TU::i(6), TU::i(8)], is_even_int),
                Ok(0),
            ),
            (
                (vec![TU::i(1), TU::i(3), TU::i(5), TU::i(7), TU::i(9)], is_even_int),
                Err(ErrorKind::ItemNotFound),
            ),
            (
                (vec![TU::i(0), TU::i(2), MetaVal::Bul(false), TU::i(6), TU::i(8)], is_even_int),
                Ok(0),
            ),
            (
                (vec![TU::i(1), TU::i(3), MetaVal::Bul(false), TU::i(7), TU::i(9)], is_even_int),
                Err(ErrorKind::NotNumeric),
            ),
        ];

        for (inputs, expected) in inputs_and_expected {
            let (input_a, input_b) = inputs;
            let produced = Impl::position_s(input_a, input_b).map_err(Into::<ErrorKind>::into);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_filter() {
        let inputs_and_expected: Vec<((_, fn(&MetaVal) -> Result<bool, Error>), _)> = vec![
            (
                (vec![], is_boolean),
                vec![],
            ),
            (
                (TU::core_nested_sequence().into_iter().map(Result::Ok).collect(), is_boolean),
                vec![Ok(TU::sample_boolean())],
            ),
            (
                (vec![Ok(MetaVal::Bul(false)), Ok(MetaVal::Int(1)), Err(Error::Sentinel)], is_boolean),
                vec![Ok(MetaVal::Bul(false)), Err(ErrorKind::Sentinel)],
            ),
            (
                (vec![Err(Error::Sentinel), Ok(MetaVal::Bul(false)), Ok(MetaVal::Int(1))], is_boolean),
                vec![Err(ErrorKind::Sentinel), Ok(MetaVal::Bul(false))],
            ),
            (
                (vec![Ok(TU::i(0)), Ok(TU::i(2)), Ok(TU::i(4)), Ok(TU::i(6)), Ok(TU::i(8))], is_even_int),
                vec![Ok(TU::i(0)), Ok(TU::i(2)), Ok(TU::i(4)), Ok(TU::i(6)), Ok(TU::i(8))],
            ),
            (
                (vec![Ok(TU::i(0)), Ok(TU::i(2)), Ok(TU::i(5)), Ok(TU::i(6)), Ok(TU::i(8))], is_even_int),
                vec![Ok(TU::i(0)), Ok(TU::i(2)), Ok(TU::i(6)), Ok(TU::i(8))],
            ),
            (
                (vec![Ok(TU::i(1)), Ok(TU::i(3)), Ok(TU::i(5)), Ok(TU::i(7)), Ok(TU::i(9))], is_even_int),
                vec![],
            ),
            (
                (vec![Ok(TU::i(0)), Ok(TU::i(2)), Ok(MetaVal::Bul(false)), Ok(TU::i(6)), Ok(TU::i(8))], is_even_int),
                vec![Ok(TU::i(0)), Ok(TU::i(2)), Err(ErrorKind::NotNumeric), Ok(TU::i(6)), Ok(TU::i(8))],
            ),
            (
                (vec![Ok(TU::i(1)), Ok(TU::i(3)), Ok(MetaVal::Bul(false)), Ok(TU::i(7)), Ok(TU::i(9))], is_even_int),
                vec![Err(ErrorKind::NotNumeric)],
            ),
        ];

        for (inputs, expected) in inputs_and_expected {
            let (input_a, input_b) = inputs;
            let produced = Impl::filter(Raw::new(input_a), input_b).map(|e| e.map_err(Into::<ErrorKind>::into)).collect::<Vec<_>>();
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_filter_s() {
        let inputs_and_expected: Vec<((_, fn(&MetaVal) -> Result<bool, Error>), _)> = vec![
            (
                (vec![], is_boolean),
                Ok(vec![]),
            ),
            (
                (TU::core_nested_sequence(), is_boolean),
                Ok(vec![TU::sample_boolean()]),
            ),
            (
                (vec![MetaVal::Bul(false), MetaVal::Int(1)], is_boolean),
                Ok(vec![MetaVal::Bul(false)]),
            ),
            (
                (vec![MetaVal::Bul(false), MetaVal::Int(1)], is_boolean),
                Ok(vec![MetaVal::Bul(false)]),
            ),
            (
                (vec![TU::i(0), TU::i(2), TU::i(4), TU::i(6), TU::i(8)], is_even_int),
                Ok(vec![TU::i(0), TU::i(2), TU::i(4), TU::i(6), TU::i(8)]),
            ),
            (
                (vec![TU::i(0), TU::i(2), TU::i(5), TU::i(6), TU::i(8)], is_even_int),
                Ok(vec![TU::i(0), TU::i(2), TU::i(6), TU::i(8)]),
            ),
            (
                (vec![TU::i(1), TU::i(3), TU::i(5), TU::i(7), TU::i(9)], is_even_int),
                Ok(vec![]),
            ),
            (
                (vec![TU::i(0), TU::i(2), MetaVal::Bul(false), TU::i(6), TU::i(8)], is_even_int),
                Err(ErrorKind::NotNumeric),
            ),
            (
                (vec![TU::i(1), TU::i(3), MetaVal::Bul(false), TU::i(7), TU::i(9)], is_even_int),
                Err(ErrorKind::NotNumeric),
            ),
        ];

        for (inputs, expected) in inputs_and_expected {
            let (input_a, input_b) = inputs;
            let produced = Impl::filter_s(input_a, input_b).map_err(Into::<ErrorKind>::into);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_map() {
        let inputs_and_expected: Vec<((_, fn(MetaVal) -> Result<MetaVal, Error>), _)> = vec![
            (
                (vec![], conv_repr),
                vec![],
            ),
            (
                (TU::core_nested_sequence().into_iter().map(Result::Ok).collect(), conv_repr),
                vec![
                    Ok(TU::s("string")), Ok(TU::s("integer")), Ok(TU::s("decimal")), Ok(TU::s("boolean")),
                    Ok(TU::s("null")), Ok(TU::s("sequence")), Ok(TU::s("mapping")),
                ],
            ),
            (
                (vec![Ok(MetaVal::Bul(false)), Ok(MetaVal::Int(1)), Err(Error::Sentinel)], conv_repr),
                vec![Ok(TU::s("boolean")), Ok(TU::s("integer")), Err(ErrorKind::Sentinel)],
            ),
            (
                (vec![Err(Error::Sentinel), Ok(MetaVal::Bul(false)), Ok(MetaVal::Int(1))], conv_repr),
                vec![Err(ErrorKind::Sentinel), Ok(TU::s("boolean")), Ok(TU::s("integer"))],
            ),
            (
                (vec![Ok(TU::i(0)), Ok(TU::i(2)), Ok(TU::i(4)), Ok(TU::i(6)), Ok(TU::i(8))], conv_add_3),
                vec![Ok(TU::i(0+3)), Ok(TU::i(2+3)), Ok(TU::i(4+3)), Ok(TU::i(6+3)), Ok(TU::i(8+3))],
            ),
            (
                (vec![Ok(TU::i(0)), Ok(TU::i(2)), Ok(MetaVal::Bul(false)), Ok(TU::i(6)), Ok(TU::i(8))], conv_add_3),
                vec![Ok(TU::i(0+3)), Ok(TU::i(2+3)), Err(ErrorKind::NotNumeric), Ok(TU::i(6+3)), Ok(TU::i(8+3))],
            ),
        ];

        for (inputs, expected) in inputs_and_expected {
            let (input_a, input_b) = inputs;
            let produced = Impl::map(Raw::new(input_a), input_b).map(|e| e.map_err(Into::<ErrorKind>::into)).collect::<Vec<_>>();
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_map_s() {
        let inputs_and_expected: Vec<((_, fn(MetaVal) -> Result<MetaVal, Error>), _)> = vec![
            (
                (vec![], conv_repr),
                Ok(vec![]),
            ),
            (
                (TU::core_nested_sequence(), conv_repr),
                Ok(vec![
                    TU::s("string"), TU::s("integer"), TU::s("decimal"), TU::s("boolean"),
                    TU::s("null"), TU::s("sequence"), TU::s("mapping"),
                ]),
            ),
            (
                (vec![MetaVal::Bul(false), MetaVal::Int(1)], conv_repr),
                Ok(vec![TU::s("boolean"), TU::s("integer")]),
            ),
            (
                (vec![TU::i(0), TU::i(2), TU::i(4), TU::i(6), TU::i(8)], conv_add_3),
                Ok(vec![TU::i(0+3), TU::i(2+3), TU::i(4+3), TU::i(6+3), TU::i(8+3)]),
            ),
            (
                (vec![TU::i(0), TU::i(2), MetaVal::Bul(false), TU::i(6), TU::i(8)], conv_add_3),
                Err(ErrorKind::NotNumeric),
            ),
        ];

        for (inputs, expected) in inputs_and_expected {
            let (input_a, input_b) = inputs;
            let produced = Impl::map_s(input_a, input_b).map_err(Into::<ErrorKind>::into);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_step_by() {
        let inputs_and_expected = vec![
            (
                (vec![], 1),
                Ok(vec![]),
            ),
            (
                (vec![], 2),
                Ok(vec![]),
            ),
            (
                (vec![], 0),
                Err(ErrorKind::ZeroStepSize),
            ),
            (
                (TU::core_nested_sequence().into_iter().map(Result::Ok).collect(), 1),
                Ok(TU::core_nested_sequence().into_iter().map(Result::Ok).collect()),
            ),
            (
                (TU::core_nested_sequence().into_iter().map(Result::Ok).collect(), 2),
                Ok(TU::core_nested_sequence().into_iter().step_by(2).map(Result::Ok).collect()),
            ),
            (
                (vec![Ok(MetaVal::Bul(false)), Ok(MetaVal::Int(1)), Err(Error::Sentinel)], 10),
                Ok(vec![Ok(MetaVal::Bul(false)), Err(ErrorKind::Sentinel)]),
            ),
            (
                // TODO: Does this case make sense?
                //       Emitting leading errors, but not counting them as "stepped", and then emitting the first non-error item.
                (vec![Err(Error::Sentinel), Ok(MetaVal::Bul(false)), Ok(MetaVal::Int(1))], 10),
                Ok(vec![Err(ErrorKind::Sentinel), Ok(MetaVal::Bul(false))]),
            ),
            (
                ((0i64..=100).into_iter().map(TU::i).map(Result::Ok).collect(), 0),
                Err(ErrorKind::ZeroStepSize),
            ),
            (
                ((0i64..=100).into_iter().map(TU::i).map(Result::Ok).collect(), 1),
                Ok((0i64..=100).into_iter().step_by(1).map(TU::i).map(Result::Ok).collect()),
            ),
            (
                ((0i64..=100).into_iter().map(TU::i).map(Result::Ok).collect(), 2),
                Ok((0i64..=100).into_iter().step_by(2).map(TU::i).map(Result::Ok).collect()),
            ),
            (
                ((0i64..=100).into_iter().map(TU::i).map(Result::Ok).collect(), 4),
                Ok((0i64..=100).into_iter().step_by(4).map(TU::i).map(Result::Ok).collect()),
            ),
        ];

        for (inputs, expected) in inputs_and_expected {
            let (input_a, input_b) = inputs;
            let produced = Impl::step_by(Raw::new(input_a), input_b).map_err(Into::<ErrorKind>::into).map(|it| it.map(|r| r.map_err(Into::<ErrorKind>::into)).collect::<Vec<_>>());
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_step_by_s() {
        let inputs_and_expected = vec![
            (
                (vec![], 1),
                Ok(vec![]),
            ),
            (
                (vec![], 2),
                Ok(vec![]),
            ),
            (
                (vec![], 0),
                Err(ErrorKind::ZeroStepSize),
            ),
            (
                (TU::core_nested_sequence(), 1),
                Ok(TU::core_nested_sequence()),
            ),
            (
                (TU::core_nested_sequence(), 2),
                Ok(TU::core_nested_sequence().into_iter().step_by(2).collect()),
            ),
            (
                (vec![MetaVal::Bul(false), MetaVal::Int(1)], 10),
                Ok(vec![MetaVal::Bul(false)]),
            ),
            (
                ((0i64..=100).into_iter().map(TU::i).collect(), 0),
                Err(ErrorKind::ZeroStepSize),
            ),
            (
                ((0i64..=100).into_iter().map(TU::i).collect(), 1),
                Ok((0i64..=100).into_iter().step_by(1).map(TU::i).collect()),
            ),
            (
                ((0i64..=100).into_iter().map(TU::i).collect(), 2),
                Ok((0i64..=100).into_iter().step_by(2).map(TU::i).collect()),
            ),
            (
                ((0i64..=100).into_iter().map(TU::i).collect(), 4),
                Ok((0i64..=100).into_iter().step_by(4).map(TU::i).collect()),
            ),
        ];

        for (inputs, expected) in inputs_and_expected {
            let (input_a, input_b) = inputs;
            let produced = Impl::step_by_s(input_a, input_b).map_err(Into::<ErrorKind>::into);
            assert_eq!(expected, produced);
        }
    }
}
