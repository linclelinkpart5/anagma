use metadata::resolver::streams::Stream;
use metadata::types::MetaVal;

pub enum Operand<'k, 'p, 's> {
    Stream(Stream<'k, 'p, 's>),
    Value(MetaVal),
}

pub struct OperandStack<'k, 'p, 's>(Vec<Operand<'k, 'p, 's>>);

#[derive(Copy, Clone, Debug)]
pub enum NullaryOp {
    Parents,
    Children,
}
