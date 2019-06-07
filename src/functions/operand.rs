use crate::metadata::types::MetaVal;
use crate::functions::util::value_producer::ValueProducer;

/// Values that are pushed onto an operand stack.
/// In order for a stack to be valid, it must result in exactly one value operand after processing.
pub enum Operand<'o> {
    Producer(ValueProducer<'o>),
    Value(MetaVal<'o>),
}
