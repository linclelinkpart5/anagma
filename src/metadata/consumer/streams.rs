use std::collections::VecDeque;

use metadata::types::MetaVal;
use metadata::producer::value::SimpleMetaValueProducer;

/// A stream is a generalization of the different kinds of lazy sequences that can be used/produced by consumers.
pub enum Stream<'k, 'p, 's> {
    Raw(SimpleMetaValueProducer<'k, 'p, 's>),
    Flatten,
    Dedup,
    Unique,
}

// struct FlattenStream<'k, 'p, 's>(Stream<'k, 'p, 's>, VecDeque<MetaVal>);

// impl<'k, 'p, 's> Iterator for FlattenStream<'k, 'p, 's> {
//     type Item = MetaVal;

//     fn next(&mut self) -> Option<Self::Item> {
//         match self.1.pop_front() {
//             Some(mv) => Some(mv),
//             None => {
//                 // Try to get the next item from the stream.
//                 match self.0.next() {
//                     None => None,
//                     Some(mv) => {
//                         // Check if this new value needs flattening.
//                         match mv {
//                             MetaVal::Seq(seq) => {},
//                         }
//                     }
//                 }
//             },
//         }
//     }
// }
