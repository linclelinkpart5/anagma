use std::convert::TryInto;

use crate::metadata::types::MetaVal;
use crate::functions::Error;
use super::Predicate;

#[derive(Clone, Copy, Debug)]
pub enum Converter {
    Nth,
    StepBy,
    Chain,
    Zip,
    Map,
    Filter,
    SkipWhile,
    TakeWhile,
    Skip,
    Take,
    All,
    Any,
    Find,
    Position,
    Interleave,
    Intersperse,
    Chunks,
    Windows,
    Predicate(Predicate),
}

impl Converter {
    // pub fn process<'mv>(&self, mv_a: MetaVal<'mv>, mv_b: MetaVal<'mv>) -> Result<MetaVal<'mv>, Error> {
    //     match self {
    //         &Self::Nth => {
    //             let seq: Vec<_> = mv_a.try_into()?;
    //             let n: usize = mv_b.try_into()?;

    //             seq.into_iter().nth(n).ok_or(Error::EmptySequence)
    //         },
    //         &Self::StepBy => {
    //             let seq: Vec<_> = mv_a.try_into()?;
    //             let step: usize = mv_b.try_into()?;

    //             if step == 0 { return Err(Error::ZeroStepSize) }

    //             let mut c = 0;
    //             let mut new_seq = vec![];
    //             for mv in seq {
    //                 c %= step;
    //                 if c == 0 { new_seq.push(mv); }
    //                 c += 1;
    //             }

    //             Ok(MetaVal::Seq(new_seq))
    //         },
    //         &Self::Chain => {
    //             let mut seq_a: Vec<_> = mv_a.try_into()?;
    //             let seq_b: Vec<_> = mv_b.try_into()?;

    //             seq_a.extend(seq_b);

    //             Ok(MetaVal::Seq(seq_a))
    //         },
    //         &Self::Zip => {
    //             let seq_a: Vec<_> = mv_a.try_into()?;
    //             let seq_b: Vec<_> = mv_b.try_into()?;

    //             let mut new_seq = vec![];
    //             for (a, b) in seq_a.into_iter().zip(seq_b) {
    //                 new_seq.push(MetaVal::Seq(vec![a, b]));
    //             }

    //             Ok(MetaVal::Seq(new_seq))
    //         },

    //         // All predicates are implicitly converters as well.
    //         &Self::Predicate(pred) => pred.process(&mv_a, &mv_b).map(MetaVal::Bul),
    //         _ => Ok(MetaVal::Nil)
    //     }
    // }
}

#[cfg(test)]
mod tests {
}
