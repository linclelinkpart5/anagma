use std::collections::VecDeque;
use std::collections::HashSet;

use crate::metadata::stream::value::MetaValueStream;
use crate::metadata::types::MetaVal;
use crate::metadata::resolver::Error;
use crate::metadata::resolver::iterable_like::IterableLike;
use crate::metadata::resolver::ops::Operand;
use crate::metadata::resolver::ops::unary::UnaryOp;

/// A stream is a generalization of the different kinds of lazy sequences that can be used/produced by consumers.
#[derive(Debug)]
pub enum Stream<'s> {
    Raw(MetaValueStream<'s>),
    Fixed(std::vec::IntoIter<MetaVal<'s>>),

    Flatten(FlattenStream<'s>),
    Dedup(DedupStream<'s>),
    Unique(UniqueStream<'s>),
    StepBy(StepByStream<'s>),
    Chain(ChainStream<'s>),
    Zip(ZipStream<'s>),
    Map(MapStream<'s>),
}

impl<'s> Stream<'s> {
    pub fn into_operand(self, collect: bool) -> Result<Operand<'s>, Error> {
        if collect {
            Ok(Operand::Value(MetaVal::Seq(self.collect::<Result<Vec<_>, _>>()?)))
        }
        else {
            Ok(Operand::Stream(self))
        }
    }
}

type StreamResult<'s> = Result<MetaVal<'s>, Error>;

impl<'s> Iterator for Stream<'s> {
    type Item = StreamResult<'s>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            &mut Self::Raw(ref mut it) => it.next().map(|res| res.map(|(_, mv)| mv)).map(|res| res.map_err(Error::ValueStream)),
            &mut Self::Fixed(ref mut it) => it.next().map(Result::Ok),

            &mut Self::Flatten(ref mut it) => it.next(),
            &mut Self::Dedup(ref mut it) => it.next(),
            &mut Self::Unique(ref mut it) => it.next(),
            &mut Self::StepBy(ref mut it) => it.next(),
            &mut Self::Chain(ref mut it) => it.next(),
            &mut Self::Zip(ref mut it) => it.next(),
            &mut Self::Map(ref mut it) => it.next(),
        }
    }
}

impl<'s> From<IterableLike<'s>> for Stream<'s> {
    fn from(il: IterableLike<'s>) -> Self {
        match il {
            IterableLike::Sequence(seq) => Stream::Fixed(seq.into_iter()),
            IterableLike::Stream(stm) => stm,
        }
    }
}

#[derive(Debug)]
pub struct FlattenStream<'s>(Box<Stream<'s>>, VecDeque<MetaVal<'s>>);

impl<'s> FlattenStream<'s> {
    pub fn new(s: Stream<'s>) -> Self {
        Self(Box::new(s), VecDeque::new())
    }
}

impl<'s> Iterator for FlattenStream<'s> {
    type Item = StreamResult<'s>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.1.pop_front() {
            Some(mv) => Some(Ok(mv)),
            None => {
                // Try to get the next item from the stream.
                match self.0.next()? {
                    Ok(MetaVal::Seq(seq)) => {
                        // Move all elements in the sequence into the queue.
                        self.1.extend(seq);
                        self.next()
                    },
                    o => Some(o),
                }
            },
        }
    }
}

#[derive(Debug)]
pub struct DedupStream<'s>(Box<Stream<'s>>, Option<MetaVal<'s>>);

impl<'s> DedupStream<'s> {
    pub fn new(s: Stream<'s>) -> Self {
        Self(Box::new(s), None)
    }
}

impl<'s> Iterator for DedupStream<'s> {
    type Item = StreamResult<'s>;

    fn next(&mut self) -> Option<Self::Item> {
        let res = self.0.next()?;

        match res {
            Err(err) => Some(Err(err)),
            Ok(curr_val) => {
                if Some(&curr_val) != self.1.as_ref() {
                    // A non-duplicate was found.
                    self.1 = Some(curr_val.clone());
                    Some(Ok(curr_val))
                }
                else {
                    // Delegate to the next call.
                    self.next()
                }
            },
        }
    }
}

#[derive(Debug)]
pub struct UniqueStream<'s>(Box<Stream<'s>>, HashSet<MetaVal<'s>>);

impl<'s> UniqueStream<'s> {
    pub fn new(s: Stream<'s>) -> Self {
        Self(Box::new(s), HashSet::new())
    }
}

impl<'s> Iterator for UniqueStream<'s> {
    type Item = StreamResult<'s>;

    fn next(&mut self) -> Option<Self::Item> {
        let res = self.0.next()?;

        match res {
            Err(err) => Some(Err(err)),
            Ok(curr_val) => {
                if self.1.contains(&curr_val) {
                    // Skip and delegate to the next call.
                    self.next()
                }
                else {
                    self.1.insert(curr_val.clone());
                    Some(Ok(curr_val))
                }
            },
        }
    }
}

#[derive(Debug)]
pub struct StepByStream<'s> {
    stream: Box<Stream<'s>>,
    curr: usize,
    n: usize,
}

impl<'s> StepByStream<'s> {
    // Can fail if step size is zero.
    pub fn new(s: Stream<'s>, n: usize) -> Result<Self, Error> {
        if n == 0 { Err(Error::ZeroStepSize) }
        else {
            Ok(Self {
                stream: Box::new(s),
                curr: n,
                n,
            })
        }
    }
}

impl<'s> Iterator for StepByStream<'s> {
    type Item = StreamResult<'s>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.stream.next()? {
            // Always report errors, even if they would not normally be "hit".
            Err(err) => Some(Err(err)),
            Ok(mv) => {
                // Output the meta value if currently at a step point.
                if self.curr >= self.n {
                    self.curr = 1;
                    Some(Ok(mv))
                }
                else {
                    self.curr += 1;
                    self.next()
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct ChainStream<'s>(Box<Stream<'s>>, Box<Stream<'s>>, bool);

impl<'s> ChainStream<'s> {
    pub fn new(s_a: Stream<'s>, s_b: Stream<'s>) -> Self {
        Self(Box::new(s_a), Box::new(s_b), false)
    }
}

impl<'s> Iterator for ChainStream<'s> {
    type Item = StreamResult<'s>;

    fn next(&mut self) -> Option<Self::Item> {
        // Iterate the first stream.
        if !self.2 {
            match self.0.next() {
                None => {
                    self.2 = true;
                    self.next()
                }
                Some(res) => Some(res),
            }
        }
        // Iterate the second stream.
        else {
            self.1.next()
        }
    }
}

#[derive(Debug)]
pub struct ZipStream<'s>(Box<Stream<'s>>, Box<Stream<'s>>);

impl<'s> ZipStream<'s> {
    pub fn new(s_a: Stream<'s>, s_b: Stream<'s>) -> Self {
        Self(Box::new(s_a), Box::new(s_b))
    }
}

impl<'s> Iterator for ZipStream<'s> {
    type Item = StreamResult<'s>;

    fn next(&mut self) -> Option<Self::Item> {
        let res_a = self.0.next()?;
        let res_b = self.1.next()?;

        match (res_a, res_b) {
            (Err(e_a), _) => Some(Err(e_a)),
            (_, Err(e_b)) => Some(Err(e_b)),
            (Ok(a), Ok(b)) => Some(Ok(MetaVal::Seq(vec![a, b]))),
        }
    }
}

impl<'s> std::iter::FusedIterator for ZipStream<'s> {}

#[derive(Debug)]
pub struct MapStream<'s>(Box<Stream<'s>>, UnaryOp);

impl<'s> MapStream<'s> {
    pub fn new(s: Stream<'s>, op: UnaryOp) -> Self {
        Self(Box::new(s), op)
    }
}

impl<'s> Iterator for MapStream<'s> {
    type Item = StreamResult<'s>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(match self.0.next()? {
            Ok(mv) => self.1.process_as_converter(Operand::Value(mv)),
            Err(err) => Err(err),
        })
    }
}
