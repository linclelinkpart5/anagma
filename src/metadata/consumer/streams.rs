use std::collections::VecDeque;
use std::collections::HashSet;

use metadata::stream::value::SimpleMetaValueStream as RawStream;
use metadata::types::MetaVal;

/// A stream is a generalization of the different kinds of lazy sequences that can be used/produced by consumers.
pub enum Stream<'k, 'p, 's> {
    Raw(RawStream<'k, 'p, 's>),
    Flatten(FlattenStream<'k, 'p, 's>),
    Dedup(DedupStream<'k, 'p, 's>),
    Unique(UniqueStream<'k, 'p, 's>),
}

impl<'k, 'p, 's> Iterator for Stream<'k, 'p, 's> {
    type Item = MetaVal;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            &mut Self::Raw(ref mut it) => it.next(),
            &mut Self::Flatten(ref mut it) => it.next(),
            &mut Self::Dedup(ref mut it) => it.next(),
            &mut Self::Unique(ref mut it) => it.next(),
        }
    }
}

pub struct FlattenStream<'k, 'p, 's>(Box<Stream<'k, 'p, 's>>, VecDeque<MetaVal>);

impl<'k, 'p, 's> Iterator for FlattenStream<'k, 'p, 's> {
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

pub struct DedupStream<'k, 'p, 's>(Box<Stream<'k, 'p, 's>>, Option<MetaVal>);

impl<'k, 'p, 's> Iterator for DedupStream<'k, 'p, 's> {
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

pub struct UniqueStream<'k, 'p, 's>(Box<Stream<'k, 'p, 's>>, HashSet<MetaVal>);

impl<'k, 'p, 's> Iterator for UniqueStream<'k, 'p, 's> {
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
