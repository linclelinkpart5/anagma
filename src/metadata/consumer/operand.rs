use metadata::consumer::streams::Stream;
use metadata::types::MetaVal;

pub enum Operand<'k, 'p, 's> {
    Stream(Stream<'k, 'p, 's>),
    Value(MetaVal),
}
