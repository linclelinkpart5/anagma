///! Wrapper type for items on the consumer stack that behave as a sequence of meta values.

use metadata::consumer::StackItem;
use metadata::types::MetaVal;
use metadata::streams::value::SimpleMetaValueStream;

pub enum IterableLike<'k, 'p, 's> {
    Stream(SimpleMetaValueStream<'k, 'p, 's>),
    Sequence(Vec<MetaVal>),
}

impl<'k, 'p, 's> From<IterableLike<'k, 'p, 's>> for StackItem<'k, 'p, 's> {
    fn from(il: IterableLike<'k, 'p, 's>) -> Self {
        match il {
            IterableLike::Stream(stream) => Self::Stream(stream),
            IterableLike::Sequence(sequence) => Self::Value(MetaVal::Seq(sequence)),
        }
    }
}
