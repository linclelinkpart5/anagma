use crate::metadata::types::MetaVal;
use crate::scripting::expr::Op1;
use crate::scripting::expr::Op2;
use crate::scripting::util::value_producer::ValueProducer;

pub enum Token<'v> {
    Producer(ValueProducer<'v>),
    Value(MetaVal),
    Op1(Op1),
    Op2(Op2),
}
