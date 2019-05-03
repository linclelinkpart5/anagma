use std::collections::VecDeque;
use std::collections::HashSet;

use crate::functions::Error;
use crate::functions::op::operator::UnaryOp;
use crate::functions::op::operator::UnaryPredicate;
use crate::functions::op::operator::UnaryConverter;
use crate::functions::op::operand::Operand;
use crate::metadata::stream::value::MetaValueStream;
use crate::metadata::types::MetaVal;

#[derive(Debug)]
pub enum StreamAdaptor<'s> {
    Raw(MetaValueStream<'s>),
    Fixed(std::vec::IntoIter<MetaVal<'s>>),

    Flatten(FlattenAdaptor<'s>),
    Dedup(DedupAdaptor<'s>),
    Unique(UniqueAdaptor<'s>),

    Filter(FilterAdaptor<'s>),
    Map(MapAdaptor<'s>),
}

impl<'s> Iterator for StreamAdaptor<'s> {
    type Item = Result<MetaVal<'s>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            &mut Self::Raw(ref mut it) => it.next().map(|res| res.map(|(_, mv)| mv).map_err(Error::ValueStream)),
            &mut Self::Fixed(ref mut it) => it.next().map(Result::Ok),

            &mut Self::Flatten(ref mut it) => it.next(),
            &mut Self::Dedup(ref mut it) => it.next(),
            &mut Self::Unique(ref mut it) => it.next(),

            &mut Self::Filter(ref mut it) => it.next(),
            &mut Self::Map(ref mut it) => it.next(),
        }
    }
}

#[derive(Debug)]
pub struct FlattenAdaptor<'s>(Box<StreamAdaptor<'s>>, VecDeque<MetaVal<'s>>);

impl<'s> FlattenAdaptor<'s> {
    pub fn new(s: StreamAdaptor<'s>) -> Self {
        Self(Box::new(s), VecDeque::new())
    }
}

impl<'s> Iterator for FlattenAdaptor<'s> {
    type Item = Result<MetaVal<'s>, Error>;

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
}#[derive(Debug)]
pub struct DedupAdaptor<'s>(Box<StreamAdaptor<'s>>, Option<MetaVal<'s>>);

impl<'s> DedupAdaptor<'s> {
    pub fn new(s: StreamAdaptor<'s>) -> Self {
        Self(Box::new(s), None)
    }
}

impl<'s> Iterator for DedupAdaptor<'s> {
    type Item = Result<MetaVal<'s>, Error>;

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
pub struct UniqueAdaptor<'s>(Box<StreamAdaptor<'s>>, HashSet<MetaVal<'s>>);

impl<'s> UniqueAdaptor<'s> {
    pub fn new(s: StreamAdaptor<'s>) -> Self {
        Self(Box::new(s), HashSet::new())
    }
}

impl<'s> Iterator for UniqueAdaptor<'s> {
    type Item = Result<MetaVal<'s>, Error>;

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
pub struct FilterAdaptor<'s>(Box<StreamAdaptor<'s>>, UnaryPredicate);

impl<'s> FilterAdaptor<'s> {
    pub fn new(s: StreamAdaptor<'s>, pred: UnaryPredicate) -> Self {
        Self(Box::new(s), pred)
    }
}

impl<'s> Iterator for FilterAdaptor<'s> {
    type Item = Result<MetaVal<'s>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.next()? {
            Ok(mv) => {
                match self.1.process(&mv) {
                    Err(err) => Some(Err(err)),
                    Ok(b) => {
                        if b { Some(Ok(mv)) }
                        else { self.next() }
                    },
                }
            },
            Err(err) => Some(Err(err)),
        }
    }
}

#[derive(Debug)]
pub struct MapAdaptor<'s>(Box<StreamAdaptor<'s>>, UnaryConverter);

impl<'s> MapAdaptor<'s> {
    pub fn new(s: StreamAdaptor<'s>, conv: UnaryConverter) -> Self {
        Self(Box::new(s), conv)
    }
}

impl<'s> Iterator for MapAdaptor<'s> {
    type Item = Result<MetaVal<'s>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.next()? {
            Ok(mv) => Some(self.1.process(mv)),
            Err(err) => Some(Err(err)),
        }
    }
}
