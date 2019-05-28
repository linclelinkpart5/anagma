pub mod unary;
pub mod binary;

pub use self::unary::Converter as UnaryConverter;
pub use self::unary::Predicate as UnaryPredicate;
pub use self::unary::IterConsumer as UnaryIterConsumer;
pub use self::unary::IterAdaptor as UnaryIterAdaptor;
pub use self::binary::Converter as BinaryConverter;
pub use self::binary::Predicate as BinaryPredicate;
pub use self::binary::IterConsumer as BinaryIterConsumer;
pub use self::binary::IterAdaptor as BinaryIterAdaptor;

use std::convert::TryFrom;
use std::convert::TryInto;

use crate::functions::Error;
use crate::metadata::types::MetaVal;

impl<'mv> TryFrom<MetaVal<'mv>> for Vec<MetaVal<'mv>> {
    type Error = Error;

    fn try_from(mv: MetaVal<'mv>) -> Result<Self, Self::Error> {
        match mv {
            MetaVal::Seq(seq) => Ok(seq),
            _ => Err(Error::NotSequence),
        }
    }
}

impl<'mv> TryFrom<&'mv MetaVal<'mv>> for &'mv Vec<MetaVal<'mv>> {
    type Error = Error;

    fn try_from(mv: &'mv MetaVal<'mv>) -> Result<Self, Self::Error> {
        match mv {
            &MetaVal::Seq(ref seq) => Ok(seq),
            _ => Err(Error::NotSequence),
        }
    }
}

// impl<'a> TryFrom<MetaVal<'a>> for usize {
//     type Error = Error;

//     fn try_from(value: MetaVal<'a>) -> Result<Self, Self::Error> {
//         match value {
//             MetaVal::Int(i) => i.try_into().map_err(|_| Error::NotIndex),
//             _ => Err(Error::NotIndex),
//         }
//     }
// }
