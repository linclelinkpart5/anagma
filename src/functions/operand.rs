use std::convert::TryInto;

use crate::metadata::types::MetaVal;
use crate::functions::util::value_producer::ValueProducer;
use crate::functions::Error;

/// Values that are pushed onto an operand stack.
/// In order for a stack to be valid, it must result in exactly one value operand after processing.
#[derive(Debug)]
pub enum Operand<'o, VP>
where
    VP: ValueProducer<'o>,
{
    Producer(VP),
    Value(MetaVal<'o>),
}
