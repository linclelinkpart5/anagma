use std::collections::VecDeque;
use std::collections::HashSet;
use std::iter::FusedIterator;

use crate::metadata::stream::value::MetaValueStream;
use crate::metadata::types::MetaVal;
use crate::functions::Error;

type Adaptor<'v> = Iterator<Item = Result<MetaVal<'v>, Error>>;

pub struct Raw<'v>(MetaValueStream<'v>);

impl<'v> Raw<'v> {
    pub fn new(mvs: MetaValueStream<'v>) -> Self {
        Self(mvs)
    }
}

impl<'v> Iterator for Raw<'v> {
    type Item = Result<MetaVal<'v>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|res| res.map(|(_, mv)| mv).map_err(Error::ValueStream))
    }
}

pub struct Fixed<'v>(std::vec::IntoIter<MetaVal<'v>>);

impl<'v> Fixed<'v> {
    pub fn new(v: Vec<MetaVal<'v>>) -> Self {
        Self(v.into_iter())
    }
}

impl<'v> Iterator for Fixed<'v> {
    type Item = Result<MetaVal<'v>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(Result::Ok)
    }
}

#[derive(Debug)]
pub struct Flatten<'v, I>(I, VecDeque<MetaVal<'v>>);

impl<'v, I> Flatten<'v, I>
where
    I: Iterator<Item = Result<MetaVal<'v>, Error>>,
{
    pub fn new(i: I) -> Self {
        Self(i, VecDeque::new())
    }
}

impl<'v, I> Iterator for Flatten<'v, I>
where
    I: Iterator<Item = Result<MetaVal<'v>, Error>>,
{
    type Item = Result<MetaVal<'v>, Error>;

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
