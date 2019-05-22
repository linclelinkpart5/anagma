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
use crate::functions::operator::UnaryConverter;
use crate::functions::operator::UnaryPredicate;
use crate::functions::util::value_producer::ValueProducer;
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

    fn all_any<'a, VP: ValueProducer<'a>>(vp: VP, u_pred: UnaryPredicate, flag: AllAny) -> Result<bool, Error> {
        let target = flag.target();
        for res_mv in vp {
            let mv = res_mv?;
            if u_pred.process(&mv)? == target { return Ok(target) }
        }

        Ok(!target)
    }

    pub fn all<'a, VP: ValueProducer<'a>>(vp: VP, u_pred: UnaryPredicate) -> Result<bool, Error> {
        Self::all_any(vp, u_pred, AllAny::All)
    }

    pub fn any<'a, VP: ValueProducer<'a>>(vp: VP, u_pred: UnaryPredicate) -> Result<bool, Error> {
        Self::all_any(vp, u_pred, AllAny::Any)
    }

    pub fn find<'a, VP: ValueProducer<'a>>(vp: VP, u_pred: UnaryPredicate) -> Result<Option<MetaVal<'a>>, Error> {
        for res_mv in vp {
            let mv = res_mv?;
            if u_pred.process(&mv)? { return Ok(Some(mv)) }
        }

        Ok(None)
    }

    pub fn position<'a, VP: ValueProducer<'a>>(vp: VP, u_pred: UnaryPredicate) -> Result<Option<usize>, Error> {
        let mut i = 0;
        for res_mv in vp {
            let mv = res_mv?;
            if u_pred.process(&mv)? { return Ok(Some(i)) }
            i += 1;
        }

        Ok(None)
    }

    pub fn filter<'a, VP: ValueProducer<'a>>(vp: VP, u_pred: UnaryPredicate) -> Result<Filter<VP>, Error> {
        Ok(Filter::new(vp, u_pred))
    }

    pub fn map<'a, VP: ValueProducer<'a>>(vp: VP, u_conv: UnaryConverter) -> Result<Map<VP>, Error> {
        Ok(Map::new(vp, u_conv))
    }

    pub fn step_by<'a, VP: ValueProducer<'a>>(vp: VP, step: usize) -> Result<StepBy<VP>, Error> {
        StepBy::new(vp, step)
    }

    pub fn chain<'a, VPA: ValueProducer<'a>, VPB: ValueProducer<'a>>(vp_a: VPA, vp_b: VPB) -> Result<Chain<VPA, VPB>, Error> {
        Ok(Chain::new(vp_a, vp_b))
    }

    pub fn zip<'a, VPA: ValueProducer<'a>, VPB: ValueProducer<'a>>(vp_a: VPA, vp_b: VPB) -> Result<Zip<VPA, VPB>, Error> {
        Ok(Zip::new(vp_a, vp_b))
    }

    pub fn skip<'a, VP: ValueProducer<'a>>(vp: VP, n: usize) -> Result<Skip<'a, VP>, Error> {
        Ok(Skip::new(vp, n))
    }

    pub fn take<'a, VP: ValueProducer<'a>>(vp: VP, n: usize) -> Result<Take<'a, VP>, Error> {
        Ok(Take::new(vp, n))
    }

    pub fn skip_while<'a, VP: ValueProducer<'a>>(vp: VP, u_pred: UnaryPredicate) -> Result<SkipWhile<VP>, Error> {
        Ok(SkipWhile::new(vp, u_pred))
    }

    pub fn take_while<'a, VP: ValueProducer<'a>>(vp: VP, u_pred: UnaryPredicate) -> Result<TakeWhile<VP>, Error> {
        Ok(TakeWhile::new(vp, u_pred))
    }

    pub fn intersperse<'a, VP: ValueProducer<'a>>(vp: VP, mv: MetaVal<'a>) -> Result<Intersperse<'a, VP>, Error> {
        Ok(Intersperse::new(vp, mv))
    }

    pub fn interleave<'a, VPA: ValueProducer<'a>, VPB: ValueProducer<'a>>(vp_a: VPA, vp_b: VPB) -> Result<Interleave<VPA, VPB>, Error> {
        Ok(Interleave::new(vp_a, vp_b))
    }
}
