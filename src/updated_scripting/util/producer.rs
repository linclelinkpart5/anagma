use std::collections::VecDeque;
use std::collections::HashSet;
use std::convert::TryFrom;
use std::iter::FusedIterator;
use std::borrow::Cow;

use crate::metadata::stream::value::MetaValueStream;
use crate::metadata::types::MetaVal;
use crate::updated_scripting::Error;
use crate::updated_scripting::traits::Predicate;
use crate::updated_scripting::traits::Converter;

pub struct Source<'a>(MetaValueStream<'a>);

impl<'a> Source<'a> {
    pub fn new(mvs: MetaValueStream<'a>) -> Self {
        Self(mvs)
    }
}

impl<'a> Iterator for Source<'a> {
    type Item = Result<MetaVal, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|res| res.map(|(_, mv)| mv).map_err(Error::ValueStream))
    }
}

pub struct Fixed(std::vec::IntoIter<MetaVal>);

impl Fixed {
    pub fn new(v: Vec<MetaVal>) -> Self {
        Self(v.into_iter())
    }
}

impl Iterator for Fixed {
    type Item = Result<MetaVal, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(Result::Ok)
    }
}

impl From<Vec<MetaVal>> for Fixed {
    fn from(v: Vec<MetaVal>) -> Self {
        Fixed::new(v)
    }
}

pub struct Raw(std::vec::IntoIter<Result<MetaVal, Error>>);

impl Raw {
    pub fn new(v: Vec<Result<MetaVal, Error>>) -> Self {
        Self(v.into_iter())
    }
}

impl Iterator for Raw {
    type Item = Result<MetaVal, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl From<Vec<Result<MetaVal, Error>>> for Raw {
    fn from(v: Vec<Result<MetaVal, Error>>) -> Self {
        Raw::new(v)
    }
}

pub struct Flatten<I>(I, VecDeque<MetaVal>)
where I: Iterator<Item = Result<MetaVal, Error>>;

impl<I> Flatten<I>
where
    I: Iterator<Item = Result<MetaVal, Error>>,
{
    pub fn new(iter: I) -> Self {
        Self(iter, VecDeque::new())
    }
}

impl<I> Iterator for Flatten<I>
where
    I: Iterator<Item = Result<MetaVal, Error>>,
{
    type Item = Result<MetaVal, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        // Try to pop from the holding queue first.
        match self.1.pop_front() {
            // If there is an item in the holding queue, return it and do not advance the original iterator.
            Some(mv) => Some(Ok(mv)),

            // Advance the underlying iterator, and process the item as appropriate.
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
