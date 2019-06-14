pub mod unary;
pub mod binary;

pub use self::unary::Op as UnaryOp;
pub use self::binary::Op as BinaryOp;

// use std::convert::TryFrom;

// use crate::functions::Error;
// use crate::metadata::types::MetaVal;

// impl<'mv> TryFrom<MetaVal<'mv>> for Vec<MetaVal<'mv>> {
//     type Error = Error;

//     fn try_from(mv: MetaVal<'mv>) -> Result<Self, Self::Error> {
//         match mv {
//             MetaVal::Seq(seq) => Ok(seq),
//             _ => Err(Error::NotSequence),
//         }
//     }
// }

// impl<'mv> TryFrom<&'mv MetaVal<'mv>> for &'mv Vec<MetaVal<'mv>> {
//     type Error = Error;

//     fn try_from(mv: &'mv MetaVal<'mv>) -> Result<Self, Self::Error> {
//         match mv {
//             &MetaVal::Seq(ref seq) => Ok(seq),
//             _ => Err(Error::NotSequence),
//         }
//     }
// }
