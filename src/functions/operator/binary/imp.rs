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
    pub fn nth<'a>(vp: ValueProducer<'a>, n: usize) -> Result<MetaVal<'a>, Error> {
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

    fn all_any<'a>(vp: ValueProducer<'a>, u_pred: UnaryPred, flag: AllAny) -> Result<bool, Error> {
        let target = flag.target();
        for res_mv in vp {
            let mv = res_mv?;
            if u_pred(&mv)? == target { return Ok(target) }
        }

        Ok(!target)
    }

    pub fn all<'a>(vp: ValueProducer<'a>, u_pred: UnaryPred) -> Result<bool, Error> {
        Self::all_any(vp, u_pred, AllAny::All)
    }

    pub fn all_s(seq: Vec<MetaVal>, u_pred: UnaryPred) -> Result<bool, Error> {
        Self::all_any(ValueProducer::fixed(seq), u_pred, AllAny::All)
    }

    pub fn any<'a>(vp: ValueProducer<'a>, u_pred: UnaryPred) -> Result<bool, Error> {
        Self::all_any(vp, u_pred, AllAny::Any)
    }

    pub fn any_s(seq: Vec<MetaVal>, u_pred: UnaryPred) -> Result<bool, Error> {
        Self::all_any(ValueProducer::fixed(seq), u_pred, AllAny::Any)
    }

    pub fn find<'a>(vp: ValueProducer<'a>, u_pred: UnaryPred) -> Result<MetaVal<'a>, Error> {
        for res_mv in vp {
            let mv = res_mv?;
            if u_pred(&mv)? { return Ok(mv) }
        }

        Err(Error::ItemNotFound)
    }

    pub fn find_s(seq: Vec<MetaVal>, u_pred: UnaryPred) -> Result<MetaVal, Error> {
        Self::find(ValueProducer::fixed(seq), u_pred)
    }

    pub fn position<'a>(vp: ValueProducer<'a>, u_pred: UnaryPred) -> Result<usize, Error> {
        let mut i = 0;
        for res_mv in vp {
            let mv = res_mv?;
            if u_pred(&mv)? { return Ok(i) }
            i += 1;
        }

        Err(Error::ItemNotFound)
    }

    pub fn position_s(seq: Vec<MetaVal>, u_pred: UnaryPred) -> Result<usize, Error> {
        Self::position(ValueProducer::fixed(seq), u_pred)
    }

    pub fn filter<'a>(vp: ValueProducer<'a>, u_pred: UnaryPred) -> Filter<'a> {
        Filter::new(vp, u_pred)
    }

    pub fn filter_s(seq: Vec<MetaVal>, u_pred: UnaryPred) -> Result<Vec<MetaVal>, Error> {
        // It is possible for the predicate to fail.
        Filter::new(ValueProducer::fixed(seq), u_pred).collect()
    }

    pub fn map<'a>(vp: ValueProducer<'a>, u_conv: UnaryConv) -> Map<'a> {
        Map::new(vp, u_conv)
    }

    pub fn map_s(seq: Vec<MetaVal>, u_conv: UnaryConv) -> Result<Vec<MetaVal>, Error> {
        // It is possible for the converter to fail.
        Map::new(ValueProducer::fixed(seq), u_conv).collect()
    }

    pub fn step_by<'a>(vp: ValueProducer<'a>, step: usize) -> Result<StepBy<'a>, Error> {
        StepBy::new(vp, step)
    }

    pub fn step_by_s(seq: Vec<MetaVal>, step: usize) -> Result<Vec<MetaVal>, Error> {
        // It is possible for the step by producer creation to fail.
        // NOTE: The match is not needed, but it seems desirable to make explicit that the collect cannot fail.
        match StepBy::new(ValueProducer::fixed(seq), step)?.collect::<Result<Vec<MetaVal>, _>>() {
            Err(_) => unreachable!(),
            Ok(seq) => Ok(seq),
        }
    }

    pub fn chain<'a>(vp_a: ValueProducer<'a>, vp_b: ValueProducer<'a>) -> Chain<'a> {
        Chain::new(vp_a, vp_b)
    }

    pub fn chain_s<'a>(seq_a: Vec<MetaVal<'a>>, seq_b: Vec<MetaVal<'a>>) -> Vec<MetaVal<'a>> {
        let mut seq_a = seq_a;
        seq_a.extend(seq_b);
        seq_a
    }

    pub fn zip<'a>(vp_a: ValueProducer<'a>, vp_b: ValueProducer<'a>) -> Zip<'a> {
        Zip::new(vp_a, vp_b)
    }

    pub fn zip_s<'a>(seq_a: Vec<MetaVal<'a>>, seq_b: Vec<MetaVal<'a>>) -> Vec<MetaVal<'a>> {
        // Zipping cannot fail.
        match Zip::new(ValueProducer::fixed(seq_a), ValueProducer::fixed(seq_b)).collect::<Result<Vec<MetaVal>, _>>() {
            Err(_) => unreachable!(),
            Ok(seq) => seq,
        }
    }

    pub fn skip<'a>(vp: ValueProducer<'a>, n: usize) -> Skip<'a> {
        Skip::new(vp, n)
    }

    pub fn skip_s(seq: Vec<MetaVal>, n: usize) -> Vec<MetaVal> {
        seq.into_iter().skip(n).collect()
    }

    pub fn take<'a>(vp: ValueProducer<'a>, n: usize) -> Take<'a> {
        Take::new(vp, n)
    }

    pub fn take_s(seq: Vec<MetaVal>, n: usize) -> Vec<MetaVal> {
        seq.into_iter().take(n).collect()
    }

    pub fn skip_while<'a>(vp: ValueProducer<'a>, u_pred: UnaryPred) -> SkipWhile<'a> {
        SkipWhile::new(vp, u_pred)
    }

    pub fn skip_while_s(seq: Vec<MetaVal>, u_pred: UnaryPred) -> Result<Vec<MetaVal>, Error> {
        // It is possible for the predicate to fail.
        SkipWhile::new(ValueProducer::fixed(seq), u_pred).collect()
    }

    pub fn take_while<'a>(vp: ValueProducer<'a>, u_pred: UnaryPred) -> TakeWhile<'a> {
        TakeWhile::new(vp, u_pred)
    }

    pub fn take_while_s(seq: Vec<MetaVal>, u_pred: UnaryPred) -> Result<Vec<MetaVal>, Error> {
        // It is possible for the predicate to fail.
        TakeWhile::new(ValueProducer::fixed(seq), u_pred).collect()
    }

    pub fn intersperse<'a>(vp: ValueProducer<'a>, mv: MetaVal<'a>) -> Intersperse<'a> {
        Intersperse::new(vp, mv)
    }

    pub fn intersperse_s<'a>(seq: Vec<MetaVal<'a>>, mv: MetaVal<'a>) -> Vec<MetaVal<'a>> {
        // Interspersing cannot fail.
        match Intersperse::new(ValueProducer::fixed(seq), mv).collect::<Result<Vec<MetaVal>, _>>() {
            Err(_) => unreachable!(),
            Ok(seq) => seq,
        }
    }

    pub fn interleave<'a>(vp_a: ValueProducer<'a>, vp_b: ValueProducer<'a>) -> Interleave<'a> {
        Interleave::new(vp_a, vp_b)
    }

    pub fn interleave_s<'a>(seq_a: Vec<MetaVal<'a>>, seq_b: Vec<MetaVal<'a>>) -> Vec<MetaVal<'a>> {
        // Interleaving cannot fail.
        match Interleave::new(ValueProducer::fixed(seq_a), ValueProducer::fixed(seq_b)).collect::<Result<Vec<MetaVal>, _>>() {
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
    use crate::functions::util::value_producer::ValueProducer as VP;

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

    fn is_lt_4_int(mv: &MetaVal) -> Result<bool, Error> {
        match mv {
            MetaVal::Int(i) => Ok(i < &4),
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
            let produced = Impl::nth(VP::raw(input_a), input_b).map_err(Into::<ErrorKind>::into);
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
            let produced = Impl::all(VP::raw(input_a), input_b).map_err(Into::<ErrorKind>::into);
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
            let produced = Impl::any(VP::raw(input_a), input_b).map_err(Into::<ErrorKind>::into);
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
            let produced = Impl::find(VP::raw(input_a), input_b).map_err(Into::<ErrorKind>::into);
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
            let produced = Impl::position(VP::raw(input_a), input_b).map_err(Into::<ErrorKind>::into);
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
            let produced = Impl::filter(VP::raw(input_a), input_b).map(|e| e.map_err(Into::<ErrorKind>::into)).collect::<Vec<_>>();
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
            let produced = Impl::map(VP::raw(input_a), input_b).map(|e| e.map_err(Into::<ErrorKind>::into)).collect::<Vec<_>>();
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
            let produced = Impl::step_by(VP::raw(input_a), input_b).map_err(Into::<ErrorKind>::into).map(|it| it.map(|r| r.map_err(Into::<ErrorKind>::into)).collect::<Vec<_>>());
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

    #[test]
    fn test_chain() {
        let inputs_and_expected = vec![
            (
                (vec![], vec![]),
                vec![],
            ),
            (
                (TU::core_nested_sequence().into_iter().map(Result::Ok).collect(), TU::core_flat_sequence().into_iter().map(Result::Ok).collect()),
                TU::core_nested_sequence().into_iter().chain(TU::core_flat_sequence()).map(Result::Ok).collect(),
            ),
            (
                (vec![Ok(MetaVal::Bul(false)), Err(Error::Sentinel)], vec![Err(Error::Sentinel), Ok(MetaVal::Bul(true))]),
                vec![Ok(MetaVal::Bul(false)), Err(ErrorKind::Sentinel), Err(ErrorKind::Sentinel), Ok(MetaVal::Bul(true))],
            ),
            (
                (TU::core_nested_sequence().into_iter().map(Result::Ok).collect(), vec![]),
                TU::core_nested_sequence().into_iter().map(Result::Ok).collect(),
            ),
            (
                (vec![], TU::core_nested_sequence().into_iter().map(Result::Ok).collect()),
                TU::core_nested_sequence().into_iter().map(Result::Ok).collect(),
            ),
        ];

        for (inputs, expected) in inputs_and_expected {
            let (input_a, input_b) = inputs;
            let produced = Impl::chain(VP::raw(input_a), VP::raw(input_b)).map(|e| e.map_err(Into::<ErrorKind>::into)).collect::<Vec<_>>();
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_chain_s() {
        let inputs_and_expected = vec![
            (
                (vec![], vec![]),
                vec![],
            ),
            (
                (TU::core_nested_sequence(), TU::core_flat_sequence()),
                TU::core_nested_sequence().into_iter().chain(TU::core_flat_sequence()).collect(),
            ),
            (
                (vec![MetaVal::Bul(false), TU::i(1)], vec![TU::i(1), MetaVal::Bul(true)]),
                vec![MetaVal::Bul(false), TU::i(1), TU::i(1), MetaVal::Bul(true)],
            ),
            (
                (TU::core_nested_sequence(), vec![]),
                TU::core_nested_sequence(),
            ),
            (
                (vec![], TU::core_nested_sequence()),
                TU::core_nested_sequence(),
            ),
        ];

        for (inputs, expected) in inputs_and_expected {
            let (input_a, input_b) = inputs;
            let produced = Impl::chain_s(input_a, input_b);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_zip() {
        let inputs_and_expected = vec![
            (
                (vec![], vec![]),
                vec![],
            ),
            (
                (
                    vec![Ok(TU::i(1)), Ok(TU::i(2)), Ok(TU::i(3)), Ok(TU::i(4))],
                    vec![Ok(TU::i(4)), Ok(TU::i(3)), Ok(TU::i(2)), Ok(TU::i(1))],
                ),
                vec![
                    Ok(MetaVal::Seq(vec![TU::i(1), TU::i(4)])),
                    Ok(MetaVal::Seq(vec![TU::i(2), TU::i(3)])),
                    Ok(MetaVal::Seq(vec![TU::i(3), TU::i(2)])),
                    Ok(MetaVal::Seq(vec![TU::i(4), TU::i(1)])),
                ],
            ),
            (
                (
                    vec![Ok(TU::i(1)), Ok(TU::i(2)), Ok(TU::i(3))],
                    vec![Ok(TU::i(4)), Ok(TU::i(3)), Ok(TU::i(2)), Ok(TU::i(1))],
                ),
                vec![
                    Ok(MetaVal::Seq(vec![TU::i(1), TU::i(4)])),
                    Ok(MetaVal::Seq(vec![TU::i(2), TU::i(3)])),
                    Ok(MetaVal::Seq(vec![TU::i(3), TU::i(2)])),
                ],
            ),
            (
                (
                    vec![Ok(TU::i(1)), Ok(TU::i(2)), Ok(TU::i(3)), Ok(TU::i(4))],
                    vec![Ok(TU::i(4)), Ok(TU::i(3)), Ok(TU::i(2))],
                ),
                vec![
                    Ok(MetaVal::Seq(vec![TU::i(1), TU::i(4)])),
                    Ok(MetaVal::Seq(vec![TU::i(2), TU::i(3)])),
                    Ok(MetaVal::Seq(vec![TU::i(3), TU::i(2)])),
                ],
            ),
            (
                (
                    vec![Ok(TU::i(1)), Err(Error::Sentinel), Ok(TU::i(3))],
                    vec![Err(Error::Sentinel), Ok(TU::i(2)), Ok(TU::i(3))],
                ),
                vec![
                    Err(ErrorKind::Sentinel),
                    Err(ErrorKind::Sentinel),
                    Ok(MetaVal::Seq(vec![TU::i(3), TU::i(3)])),
                ],
            ),
            (
                (
                    vec![Ok(TU::i(1)), Ok(TU::i(2)), Ok(TU::i(3))],
                    vec![Err(Error::Sentinel), Err(Error::Sentinel)],
                ),
                vec![
                    Err(ErrorKind::Sentinel),
                    Err(ErrorKind::Sentinel),
                ],
            ),
            (
                (
                    vec![],
                    vec![Ok(TU::i(4)), Ok(TU::i(3)), Ok(TU::i(2)), Ok(TU::i(1))],
                ),
                vec![],
            ),
            (
                (
                    vec![Ok(TU::i(1)), Ok(TU::i(2)), Ok(TU::i(3)), Ok(TU::i(4))],
                    vec![],
                ),
                vec![],
            ),
        ];

        for (inputs, expected) in inputs_and_expected {
            let (input_a, input_b) = inputs;
            let produced = Impl::zip(VP::raw(input_a), VP::raw(input_b)).map(|e| e.map_err(Into::<ErrorKind>::into)).collect::<Vec<_>>();
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_zip_s() {
        let inputs_and_expected = vec![
            (
                (vec![], vec![]),
                vec![],
            ),
            (
                (
                    vec![TU::i(1), TU::i(2), TU::i(3), TU::i(4)],
                    vec![TU::i(4), TU::i(3), TU::i(2), TU::i(1)],
                ),
                vec![
                    MetaVal::Seq(vec![TU::i(1), TU::i(4)]),
                    MetaVal::Seq(vec![TU::i(2), TU::i(3)]),
                    MetaVal::Seq(vec![TU::i(3), TU::i(2)]),
                    MetaVal::Seq(vec![TU::i(4), TU::i(1)]),
                ],
            ),
            (
                (
                    vec![TU::i(1), TU::i(2), TU::i(3)],
                    vec![TU::i(4), TU::i(3), TU::i(2), TU::i(1)],
                ),
                vec![
                    MetaVal::Seq(vec![TU::i(1), TU::i(4)]),
                    MetaVal::Seq(vec![TU::i(2), TU::i(3)]),
                    MetaVal::Seq(vec![TU::i(3), TU::i(2)]),
                ],
            ),
            (
                (
                    vec![TU::i(1), TU::i(2), TU::i(3), TU::i(4)],
                    vec![TU::i(4), TU::i(3), TU::i(2)],
                ),
                vec![
                    MetaVal::Seq(vec![TU::i(1), TU::i(4)]),
                    MetaVal::Seq(vec![TU::i(2), TU::i(3)]),
                    MetaVal::Seq(vec![TU::i(3), TU::i(2)]),
                ],
            ),
            (
                (
                    vec![],
                    vec![TU::i(4), TU::i(3), TU::i(2), TU::i(1)],
                ),
                vec![],
            ),
            (
                (
                    vec![TU::i(1), TU::i(2), TU::i(3), TU::i(4)],
                    vec![],
                ),
                vec![],
            ),
        ];

        for (inputs, expected) in inputs_and_expected {
            let (input_a, input_b) = inputs;
            let produced = Impl::zip_s(input_a, input_b);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_skip() {
        let inputs_and_expected = vec![
            (
                (vec![], 0),
                vec![],
            ),
            (
                (vec![], 1),
                vec![],
            ),
            (
                (vec![Ok(TU::i(1)), Ok(TU::i(2)), Ok(TU::i(3)), Ok(TU::i(4)), Ok(TU::i(5))], 0),
                vec![Ok(TU::i(1)), Ok(TU::i(2)), Ok(TU::i(3)), Ok(TU::i(4)), Ok(TU::i(5))],
            ),
            (
                (vec![Ok(TU::i(1)), Ok(TU::i(2)), Ok(TU::i(3)), Ok(TU::i(4)), Ok(TU::i(5))], 1),
                vec![Ok(TU::i(2)), Ok(TU::i(3)), Ok(TU::i(4)), Ok(TU::i(5))],
            ),
            (
                (vec![Ok(TU::i(1)), Ok(TU::i(2)), Ok(TU::i(3)), Ok(TU::i(4)), Ok(TU::i(5))], 2),
                vec![Ok(TU::i(3)), Ok(TU::i(4)), Ok(TU::i(5))],
            ),
            (
                (vec![Ok(TU::i(1)), Ok(TU::i(2)), Ok(TU::i(3)), Ok(TU::i(4)), Ok(TU::i(5))], 4),
                vec![Ok(TU::i(5))],
            ),
            (
                (vec![Ok(TU::i(1)), Ok(TU::i(2)), Ok(TU::i(3)), Ok(TU::i(4)), Ok(TU::i(5))], 8),
                vec![],
            ),
            (
                (vec![Err(Error::Sentinel), Err(Error::Sentinel), Ok(TU::i(3)), Ok(TU::i(4)), Ok(TU::i(5))], 0),
                vec![Err(ErrorKind::Sentinel), Err(ErrorKind::Sentinel), Ok(TU::i(3)), Ok(TU::i(4)), Ok(TU::i(5))],
            ),
            (
                (vec![Err(Error::Sentinel), Err(Error::Sentinel), Ok(TU::i(3)), Ok(TU::i(4)), Ok(TU::i(5))], 1),
                vec![Err(ErrorKind::Sentinel), Err(ErrorKind::Sentinel), Ok(TU::i(3)), Ok(TU::i(4)), Ok(TU::i(5))],
            ),
            (
                (vec![Err(Error::Sentinel), Err(Error::Sentinel), Ok(TU::i(3)), Ok(TU::i(4)), Ok(TU::i(5))], 2),
                vec![Err(ErrorKind::Sentinel), Err(ErrorKind::Sentinel), Ok(TU::i(3)), Ok(TU::i(4)), Ok(TU::i(5))],
            ),
            (
                (vec![Err(Error::Sentinel), Err(Error::Sentinel), Ok(TU::i(3)), Ok(TU::i(4)), Ok(TU::i(5))], 3),
                vec![Err(ErrorKind::Sentinel), Err(ErrorKind::Sentinel), Ok(TU::i(4)), Ok(TU::i(5))],
            ),
            (
                (vec![Err(Error::Sentinel), Err(Error::Sentinel), Ok(TU::i(3)), Ok(TU::i(4)), Ok(TU::i(5))], 4),
                vec![Err(ErrorKind::Sentinel), Err(ErrorKind::Sentinel), Ok(TU::i(5))],
            ),
            (
                (vec![Err(Error::Sentinel), Err(Error::Sentinel), Ok(TU::i(3)), Ok(TU::i(4)), Ok(TU::i(5))], 6),
                vec![Err(ErrorKind::Sentinel), Err(ErrorKind::Sentinel)],
            ),
        ];

        for (inputs, expected) in inputs_and_expected {
            let (input_a, input_b) = inputs;
            let produced = Impl::skip(VP::raw(input_a), input_b).map(|e| e.map_err(Into::<ErrorKind>::into)).collect::<Vec<_>>();
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_skip_s() {
        let inputs_and_expected = vec![
            (
                (vec![], 0),
                vec![],
            ),
            (
                (vec![], 1),
                vec![],
            ),
            (
                (vec![TU::i(1), TU::i(2), TU::i(3), TU::i(4), TU::i(5)], 0),
                vec![TU::i(1), TU::i(2), TU::i(3), TU::i(4), TU::i(5)],
            ),
            (
                (vec![TU::i(1), TU::i(2), TU::i(3), TU::i(4), TU::i(5)], 1),
                vec![TU::i(2), TU::i(3), TU::i(4), TU::i(5)],
            ),
            (
                (vec![TU::i(1), TU::i(2), TU::i(3), TU::i(4), TU::i(5)], 2),
                vec![TU::i(3), TU::i(4), TU::i(5)],
            ),
            (
                (vec![TU::i(1), TU::i(2), TU::i(3), TU::i(4), TU::i(5)], 4),
                vec![TU::i(5)],
            ),
            (
                (vec![TU::i(1), TU::i(2), TU::i(3), TU::i(4), TU::i(5)], 8),
                vec![],
            ),
        ];

        for (inputs, expected) in inputs_and_expected {
            let (input_a, input_b) = inputs;
            let produced = Impl::skip_s(input_a, input_b);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_take() {
        let inputs_and_expected = vec![
            (
                (vec![], 0),
                vec![],
            ),
            (
                (vec![], 1),
                vec![],
            ),
            (
                (vec![Ok(TU::i(1)), Ok(TU::i(2)), Ok(TU::i(3)), Ok(TU::i(4)), Ok(TU::i(5))], 0),
                vec![],
            ),
            (
                (vec![Ok(TU::i(1)), Ok(TU::i(2)), Ok(TU::i(3)), Ok(TU::i(4)), Ok(TU::i(5))], 1),
                vec![Ok(TU::i(1))],
            ),
            (
                (vec![Ok(TU::i(1)), Ok(TU::i(2)), Ok(TU::i(3)), Ok(TU::i(4)), Ok(TU::i(5))], 2),
                vec![Ok(TU::i(1)), Ok(TU::i(2))],
            ),
            (
                (vec![Ok(TU::i(1)), Ok(TU::i(2)), Ok(TU::i(3)), Ok(TU::i(4)), Ok(TU::i(5))], 4),
                vec![Ok(TU::i(1)), Ok(TU::i(2)), Ok(TU::i(3)), Ok(TU::i(4))],
            ),
            (
                (vec![Ok(TU::i(1)), Ok(TU::i(2)), Ok(TU::i(3)), Ok(TU::i(4)), Ok(TU::i(5))], 8),
                vec![Ok(TU::i(1)), Ok(TU::i(2)), Ok(TU::i(3)), Ok(TU::i(4)), Ok(TU::i(5))],
            ),
            (
                (vec![Err(Error::Sentinel), Err(Error::Sentinel), Ok(TU::i(3)), Ok(TU::i(4)), Ok(TU::i(5))], 0),
                vec![],
            ),
            (
                (vec![Err(Error::Sentinel), Err(Error::Sentinel), Ok(TU::i(3)), Ok(TU::i(4)), Ok(TU::i(5))], 1),
                vec![Err(ErrorKind::Sentinel)],
            ),
            (
                (vec![Err(Error::Sentinel), Err(Error::Sentinel), Ok(TU::i(3)), Ok(TU::i(4)), Ok(TU::i(5))], 2),
                vec![Err(ErrorKind::Sentinel), Err(ErrorKind::Sentinel)],
            ),
            (
                (vec![Err(Error::Sentinel), Err(Error::Sentinel), Ok(TU::i(3)), Ok(TU::i(4)), Ok(TU::i(5))], 3),
                vec![Err(ErrorKind::Sentinel), Err(ErrorKind::Sentinel), Ok(TU::i(3))],
            ),
            (
                (vec![Err(Error::Sentinel), Err(Error::Sentinel), Ok(TU::i(3)), Ok(TU::i(4)), Ok(TU::i(5))], 4),
                vec![Err(ErrorKind::Sentinel), Err(ErrorKind::Sentinel), Ok(TU::i(3)), Ok(TU::i(4))],
            ),
            (
                (vec![Err(Error::Sentinel), Err(Error::Sentinel), Ok(TU::i(3)), Ok(TU::i(4)), Ok(TU::i(5))], 6),
                vec![Err(ErrorKind::Sentinel), Err(ErrorKind::Sentinel), Ok(TU::i(3)), Ok(TU::i(4)), Ok(TU::i(5))],
            ),
        ];

        for (inputs, expected) in inputs_and_expected {
            let (input_a, input_b) = inputs;
            let produced = Impl::take(VP::raw(input_a), input_b).map(|e| e.map_err(Into::<ErrorKind>::into)).collect::<Vec<_>>();
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_take_s() {
        let inputs_and_expected = vec![
            (
                (vec![], 0),
                vec![],
            ),
            (
                (vec![], 1),
                vec![],
            ),
            (
                (vec![TU::i(1), TU::i(2), TU::i(3), TU::i(4), TU::i(5)], 0),
                vec![],
            ),
            (
                (vec![TU::i(1), TU::i(2), TU::i(3), TU::i(4), TU::i(5)], 1),
                vec![TU::i(1)],
            ),
            (
                (vec![TU::i(1), TU::i(2), TU::i(3), TU::i(4), TU::i(5)], 2),
                vec![TU::i(1), TU::i(2)],
            ),
            (
                (vec![TU::i(1), TU::i(2), TU::i(3), TU::i(4), TU::i(5)], 4),
                vec![TU::i(1), TU::i(2), TU::i(3), TU::i(4)],
            ),
            (
                (vec![TU::i(1), TU::i(2), TU::i(3), TU::i(4), TU::i(5)], 8),
                vec![TU::i(1), TU::i(2), TU::i(3), TU::i(4), TU::i(5)],
            ),
        ];

        for (inputs, expected) in inputs_and_expected {
            let (input_a, input_b) = inputs;
            let produced = Impl::take_s(input_a, input_b);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_skip_while() {
        let inputs_and_expected: Vec<((_, fn(&MetaVal) -> Result<bool, Error>), _)> = vec![
            (
                (vec![], is_lt_4_int),
                vec![],
            ),
            (
                (vec![Ok(TU::i(1)), Ok(TU::i(2)), Ok(TU::i(3))], is_lt_4_int),
                vec![],
            ),
            (
                (vec![Ok(TU::i(4)), Ok(TU::i(5)), Ok(TU::i(6))], is_lt_4_int),
                vec![Ok(TU::i(4)), Ok(TU::i(5)), Ok(TU::i(6))],
            ),
            (
                (vec![Ok(TU::i(1)), Ok(TU::i(2)), Ok(TU::i(3)), Ok(TU::i(4)), Ok(TU::i(5)), Ok(TU::i(6))], is_lt_4_int),
                vec![Ok(TU::i(4)), Ok(TU::i(5)), Ok(TU::i(6))],
            ),
            (
                (vec![Ok(TU::i(1)), Err(Error::Sentinel), Ok(TU::i(2)), Ok(TU::i(3)), Ok(TU::i(4)), Ok(TU::i(5))], is_lt_4_int),
                vec![Err(ErrorKind::Sentinel), Ok(TU::i(4)), Ok(TU::i(5))],
            ),
            (
                (vec![Ok(TU::i(1)), Ok(TU::i(2)), Ok(TU::i(3)), Ok(TU::i(4)), Ok(TU::i(5)), Err(Error::Sentinel)], is_lt_4_int),
                vec![Ok(TU::i(4)), Ok(TU::i(5)), Err(ErrorKind::Sentinel)],
            ),
            (
                (vec![Ok(TU::s("a")), Ok(TU::i(1)), Ok(TU::i(2)), Ok(TU::i(3)), Ok(TU::i(4)), Ok(TU::i(5))], is_lt_4_int),
                vec![Err(ErrorKind::NotNumeric), Ok(TU::i(4)), Ok(TU::i(5))],
            ),
            (
                (vec![Ok(TU::i(1)), Ok(TU::i(2)), Ok(TU::i(3)), Ok(TU::i(4)), Ok(TU::i(5)), Ok(TU::s("a"))], is_lt_4_int),
                vec![Ok(TU::i(4)), Ok(TU::i(5)), Ok(TU::s("a"))],
            ),
        ];

        for (inputs, expected) in inputs_and_expected {
            let (input_a, input_b) = inputs;
            let produced = Impl::skip_while(VP::raw(input_a), input_b).map(|e| e.map_err(Into::<ErrorKind>::into)).collect::<Vec<_>>();
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_skip_while_s() {
        let inputs_and_expected: Vec<((_, fn(&MetaVal) -> Result<bool, Error>), _)> = vec![
            (
                (vec![], is_lt_4_int),
                Ok(vec![]),
            ),
            (
                (vec![TU::i(1), TU::i(2), TU::i(3)], is_lt_4_int),
                Ok(vec![]),
            ),
            (
                (vec![TU::i(4), TU::i(5), TU::i(6)], is_lt_4_int),
                Ok(vec![TU::i(4), TU::i(5), TU::i(6)]),
            ),
            (
                (vec![TU::i(1), TU::i(2), TU::i(3), TU::i(4), TU::i(5), TU::i(6)], is_lt_4_int),
                Ok(vec![TU::i(4), TU::i(5), TU::i(6)]),
            ),
            (
                (vec![TU::s("a"), TU::i(1), TU::i(2), TU::i(3), TU::i(4), TU::i(5)], is_lt_4_int),
                Err(ErrorKind::NotNumeric),
            ),
            (
                (vec![TU::i(1), TU::i(2), TU::i(3), TU::i(4), TU::i(5), TU::s("a")], is_lt_4_int),
                Ok(vec![TU::i(4), TU::i(5), TU::s("a")]),
            ),
        ];

        for (inputs, expected) in inputs_and_expected {
            let (input_a, input_b) = inputs;
            let produced = Impl::skip_while_s(input_a, input_b).map_err(Into::<ErrorKind>::into);
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_take_while() {
        let inputs_and_expected: Vec<((_, fn(&MetaVal) -> Result<bool, Error>), _)> = vec![
            (
                (vec![], is_lt_4_int),
                vec![],
            ),
            (
                (vec![Ok(TU::i(1)), Ok(TU::i(2)), Ok(TU::i(3))], is_lt_4_int),
                vec![Ok(TU::i(1)), Ok(TU::i(2)), Ok(TU::i(3))],
            ),
            (
                (vec![Ok(TU::i(4)), Ok(TU::i(5)), Ok(TU::i(6))], is_lt_4_int),
                vec![],
            ),
            (
                (vec![Ok(TU::i(1)), Ok(TU::i(2)), Ok(TU::i(3)), Ok(TU::i(4)), Ok(TU::i(5)), Ok(TU::i(6))], is_lt_4_int),
                vec![Ok(TU::i(1)), Ok(TU::i(2)), Ok(TU::i(3))],
            ),
            (
                (vec![Ok(TU::i(1)), Err(Error::Sentinel), Ok(TU::i(2)), Ok(TU::i(3)), Ok(TU::i(4)), Ok(TU::i(5))], is_lt_4_int),
                vec![Ok(TU::i(1)), Err(ErrorKind::Sentinel), Ok(TU::i(2)), Ok(TU::i(3))],
            ),
            (
                (vec![Ok(TU::i(1)), Ok(TU::i(2)), Ok(TU::i(3)), Ok(TU::i(4)), Ok(TU::i(5)), Err(Error::Sentinel)], is_lt_4_int),
                vec![Ok(TU::i(1)), Ok(TU::i(2)), Ok(TU::i(3))],
            ),
            (
                (vec![Ok(TU::s("a")), Ok(TU::i(1)), Ok(TU::i(2)), Ok(TU::i(3)), Ok(TU::i(4)), Ok(TU::i(5))], is_lt_4_int),
                vec![Err(ErrorKind::NotNumeric), Ok(TU::i(1)), Ok(TU::i(2)), Ok(TU::i(3))],
            ),
            (
                (vec![Ok(TU::i(1)), Ok(TU::i(2)), Ok(TU::i(3)), Ok(TU::i(4)), Ok(TU::i(5)), Ok(TU::s("a"))], is_lt_4_int),
                vec![Ok(TU::i(1)), Ok(TU::i(2)), Ok(TU::i(3))],
            ),
        ];

        for (inputs, expected) in inputs_and_expected {
            let (input_a, input_b) = inputs;
            let produced = Impl::take_while(VP::raw(input_a), input_b).map(|e| e.map_err(Into::<ErrorKind>::into)).collect::<Vec<_>>();
            assert_eq!(expected, produced);
        }
    }

    #[test]
    fn test_take_while_s() {
        let inputs_and_expected: Vec<((_, fn(&MetaVal) -> Result<bool, Error>), _)> = vec![
            (
                (vec![], is_lt_4_int),
                Ok(vec![]),
            ),
            (
                (vec![TU::i(1), TU::i(2), TU::i(3)], is_lt_4_int),
                Ok(vec![TU::i(1), TU::i(2), TU::i(3)]),
            ),
            (
                (vec![TU::i(4), TU::i(5), TU::i(6)], is_lt_4_int),
                Ok(vec![]),
            ),
            (
                (vec![TU::i(1), TU::i(2), TU::i(3), TU::i(4), TU::i(5), TU::i(6)], is_lt_4_int),
                Ok(vec![TU::i(1), TU::i(2), TU::i(3)]),
            ),
            (
                (vec![TU::s("a"), TU::i(1), TU::i(2), TU::i(3), TU::i(4), TU::i(5)], is_lt_4_int),
                Err(ErrorKind::NotNumeric),
            ),
            (
                (vec![TU::i(1), TU::i(2), TU::i(3), TU::i(4), TU::i(5), TU::s("a")], is_lt_4_int),
                Ok(vec![TU::i(1), TU::i(2), TU::i(3)]),
            ),
        ];

        for (inputs, expected) in inputs_and_expected {
            let (input_a, input_b) = inputs;
            let produced = Impl::take_while_s(input_a, input_b).map_err(Into::<ErrorKind>::into);
            assert_eq!(expected, produced);
        }
    }
}
