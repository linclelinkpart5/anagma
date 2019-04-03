use std::collections::VecDeque;
use std::collections::HashSet;

use metadata::types::MetaVal;

/// A stream is a generalization of the different kinds of lazy sequences that can be used/produced by consumers.
pub enum Stream<I: Iterator<Item = MetaVal>> {
    Flatten(FlattenStream<I>),
    Dedup(DedupStream<I>),
    Unique(UniqueStream<I>),
}

impl<I> Iterator for Stream<I>
where
    I: Iterator<Item = MetaVal>,
{
    type Item = MetaVal;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            &mut Self::Flatten(ref mut it) => it.next(),
            &mut Self::Dedup(ref mut it) => it.next(),
            &mut Self::Unique(ref mut it) => it.next(),
        }
    }
}

pub struct FlattenStream<I: Iterator<Item = MetaVal>>(I, VecDeque<MetaVal>);

impl<I> Iterator for FlattenStream<I>
where
    I: Iterator<Item = MetaVal>,
{
    type Item = MetaVal;

    fn next(&mut self) -> Option<Self::Item> {
        match self.1.pop_front() {
            Some(mv) => Some(mv),
            None => {
                // Try to get the next item from the stream.
                match self.0.next() {
                    None => None,
                    Some(mv) => {
                        // Check if this new value needs flattening.
                        match mv {
                            MetaVal::Seq(seq) => {
                                // Move all elements in the sequence into the queue.
                                self.1.extend(seq);
                                self.next()
                            },
                            mv => Some(mv),
                        }
                    }
                }
            },
        }
    }
}

pub struct DedupStream<I: Iterator<Item = MetaVal>>(I, Option<MetaVal>);

impl<I> Iterator for DedupStream<I>
where
    I: Iterator<Item = MetaVal>,
{
    type Item = MetaVal;

    fn next(&mut self) -> Option<Self::Item> {
        let curr_val = self.0.next()?;

        if Some(&curr_val) != self.1.as_ref() {
            // A non-duplicate was found.
            self.1 = Some(curr_val.clone());
            Some(curr_val)
        }
        else {
            // Delegate to the next call.
            self.next()
        }
    }
}

pub struct UniqueStream<I: Iterator<Item = MetaVal>>(I, HashSet<MetaVal>);

impl<I> Iterator for UniqueStream<I>
where
    I: Iterator<Item = MetaVal>,
{
    type Item = MetaVal;

    fn next(&mut self) -> Option<Self::Item> {
        let curr_val = self.0.next()?;

        if self.1.contains(&curr_val) {
            // Skip and delegate to the next call.
            self.next()
        }
        else {
            self.1.insert(curr_val.clone());
            Some(curr_val)
        }
    }
}
