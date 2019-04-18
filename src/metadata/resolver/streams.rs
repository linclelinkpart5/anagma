use std::collections::VecDeque;
use std::collections::HashSet;

use crate::metadata::stream::value::MetaValueStream;
use crate::metadata::types::MetaVal;
use crate::metadata::resolver::Error;

/// A stream is a generalization of the different kinds of lazy sequences that can be used/produced by consumers.
#[derive(Debug)]
pub enum Stream<'s> {
    Raw(MetaValueStream<'s>),
    Fixed(std::vec::IntoIter<MetaVal<'s>>),

    Flatten(FlattenStream<'s>),
    Dedup(DedupStream<'s>),
    Unique(UniqueStream<'s>),
    StepBy(StepByStream<'s>),
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
pub struct StepByStream<'s>(std::iter::StepBy<Box<Stream<'s>>>);

impl<'s> StepByStream<'s> {
    pub fn new(s: Stream<'s>, n: usize) -> Self {
        Self(Box::new(s).step_by(n))
    }
}

impl<'s> Iterator for StepByStream<'s> {
    type Item = StreamResult<'s>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}
