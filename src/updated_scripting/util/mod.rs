
pub mod iterable_like;
pub mod iterator_like;
pub mod step_by_emitter;
pub mod producer;

use crate::metadata::types::MetaVal;

pub use self::iterable_like::IterableLike;
pub use self::iterator_like::IteratorLike;
pub use self::step_by_emitter::StepByEmitter;

pub struct Util;

impl Util {
    /// A sorting comparison between meta values that handles numerical comparisons intelligently.
    pub fn default_sort_by<'mv>(a: &MetaVal, b: &MetaVal) -> std::cmp::Ordering {
        // Smooth over comparsions between integers and decimals.
        // TODO: Create a stable ordering for equal integers and decimals. (e.g. I(5) vs D(5.0))
        match (a, b) {
            (&MetaVal::Int(ref i), &MetaVal::Dec(ref d)) => {
                let i_d = (*i).into();
                // NOTE: Do this to avoid having to import other modules just for type inference.
                d.cmp(&i_d).reverse()
            },
            (&MetaVal::Dec(ref d), &MetaVal::Int(ref i)) => {
                let i_d = (*i).into();
                d.cmp(&i_d)
            },
            (na, nb) => na.cmp(&nb),
        }
    }
}
