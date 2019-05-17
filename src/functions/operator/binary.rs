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
use crate::functions::util::stream_adaptor::StreamAdaptor;
use crate::functions::util::stream_adaptor::*;

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
    pub fn nth(sa: StreamAdaptor, n: usize) -> Result<MetaVal, Error> {
        let mut i = 0;
        for res_mv in sa {
            let mv = res_mv?;

            if i == n { return Ok(mv) }
            else { i += 1; }
        }

        Err(Error::OutOfBounds)
    }

    fn all_any(sa: StreamAdaptor, u_pred: UnaryPredicate, flag: AllAny) -> Result<bool, Error> {
        let target = flag.target();
        for res_mv in sa {
            let mv = res_mv?;
            if u_pred.process(&mv)? == target { return Ok(target) }
        }

        Ok(!target)
    }

    pub fn all(sa: StreamAdaptor, u_pred: UnaryPredicate) -> Result<bool, Error> {
        Self::all_any(sa, u_pred, AllAny::All)
    }

    pub fn any(sa: StreamAdaptor, u_pred: UnaryPredicate) -> Result<bool, Error> {
        Self::all_any(sa, u_pred, AllAny::Any)
    }

    pub fn find(sa: StreamAdaptor, u_pred: UnaryPredicate) -> Result<Option<MetaVal>, Error> {
        for res_mv in sa {
            let mv = res_mv?;
            if u_pred.process(&mv)? { return Ok(Some(mv)) }
        }

        Ok(None)
    }

    pub fn position(sa: StreamAdaptor, u_pred: UnaryPredicate) -> Result<Option<usize>, Error> {
        let mut i = 0;
        for res_mv in sa {
            let mv = res_mv?;
            if u_pred.process(&mv)? { return Ok(Some(i)) }
            i += 1;
        }

        Ok(None)
    }

    pub fn filter(sa: StreamAdaptor, u_pred: UnaryPredicate) -> Result<FilterAdaptor, Error> {
        Ok(FilterAdaptor::new(sa, u_pred))
    }

    pub fn map(sa: StreamAdaptor, u_conv: UnaryConverter) -> Result<MapAdaptor, Error> {
        Ok(MapAdaptor::new(sa, u_conv))
    }

    pub fn step_by(sa: StreamAdaptor, step: usize) -> Result<StepByAdaptor, Error> {
        StepByAdaptor::new(sa, step)
    }

    pub fn chain<'a>(sa_a: StreamAdaptor<'a>, sa_b: StreamAdaptor<'a>) -> Result<ChainAdaptor<'a>, Error> {
        Ok(ChainAdaptor::new(sa_a, sa_b))
    }

    pub fn zip<'a>(sa_a: StreamAdaptor<'a>, sa_b: StreamAdaptor<'a>) -> Result<ZipAdaptor<'a>, Error> {
        Ok(ZipAdaptor::new(sa_a, sa_b))
    }

    pub fn skip(sa: StreamAdaptor, n: usize) -> Result<SkipAdaptor, Error> {
        Ok(SkipAdaptor::new(sa, n))
    }

    pub fn take(sa: StreamAdaptor, n: usize) -> Result<TakeAdaptor, Error> {
        Ok(TakeAdaptor::new(sa, n))
    }

    pub fn skip_while(sa: StreamAdaptor, u_pred: UnaryPredicate) -> Result<SkipWhileAdaptor, Error> {
        Ok(SkipWhileAdaptor::new(sa, u_pred))
    }

    pub fn take_while(sa: StreamAdaptor, u_pred: UnaryPredicate) -> Result<TakeWhileAdaptor, Error> {
        Ok(TakeWhileAdaptor::new(sa, u_pred))
    }

    pub fn intersperse<'a>(sa: StreamAdaptor<'a>, mv: MetaVal<'a>) -> Result<IntersperseAdaptor<'a>, Error> {
        Ok(IntersperseAdaptor::new(sa, mv))
    }

    pub fn interleave<'a>(sa_a: StreamAdaptor<'a>, sa_b: StreamAdaptor<'a>) -> Result<InterleaveAdaptor<'a>, Error> {
        Ok(InterleaveAdaptor::new(sa_a, sa_b))
    }
}
