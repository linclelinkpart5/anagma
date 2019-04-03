mod iterable_like;
mod number_like;
mod streams;

use metadata::types::MetaVal;
use metadata::stream::value::SimpleMetaValueStream;
use metadata::stream::value::Error as ValueStreamError;

use itertools::Itertools;

#[derive(Debug)]
pub enum Error {
    FieldStream(ValueStreamError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::FieldStream(ref err) => write!(f, "field stream error: {}", err),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Self::FieldStream(ref err) => Some(err),
        }
    }
}

impl From<ValueStreamError> for Error {
    fn from(err: ValueStreamError) -> Self {
        Self::FieldStream(err)
    }
}

pub enum Operand<'k, 'p, 's> {
    Stream(SimpleMetaValueStream<'k, 'p, 's>),
    Value(MetaVal),
    // Predicate(fn(&MetaVal) -> bool),
}

pub struct OperandStack<'k, 'p, 's>(Vec<Operand<'k, 'p, 's>>);

#[derive(Copy, Clone, Debug)]
pub enum NullaryOp {
    Parents,
    Children,
}

#[derive(Copy, Clone, Debug)]
pub enum UnaryOp {
    // Input: Iterables
    Collect,
    Count,
    First,
    Last,
    Enum,
    Flatten,
    Max,
    Min,
    Rev,
    Sum,
    Product,
    Dedup,
    Unique,
    AllEqual,
    Sort,

    // Input: Numbers
    Neg,

    // Input: Strings
    Upper,
    Lower,
    Title,
    Caps,
}

/// Methods for processing and fetching useful data from field streams.
/// Unless specified, all methods only operate on valid values, and ignore errors.
pub enum FieldConsumer {
    /// Return all values from the stream.
    Collect,
    /// Count the number of values from the stream.
    Count,
    /// Return the first found value from the stream.
    First,
    /// Return the last found value from the stream.
    Last,
    /// Flattens any sequence values from the stream, leaving other meta values unchanged.
    Flatten,
    /// Filters out duplicates from consecutive runs of values.
    Dedup,
    /// Filters out values that have already been produced once.
    Unique,
    /// Equals true if all the values in the stream are equal to each other.
    AllEqual,
}

impl FieldConsumer {
    pub fn process(&self, field_stream: &mut SimpleMetaValueStream) -> MetaVal {
        match self {
            &Self::Collect => MetaVal::Seq(field_stream.collect()),
            &Self::Count => MetaVal::Int(field_stream.count() as i64),
            &Self::First => field_stream.next().unwrap_or_else(|| MetaVal::Nil),
            &Self::Last => field_stream.last().unwrap_or_else(|| MetaVal::Nil),
            &Self::Flatten => {
                let mut flat = vec![];

                for mv in field_stream {
                    match mv {
                        MetaVal::Seq(seq) => flat.extend(seq.into_iter()),
                        o => flat.push(o),
                    }
                }

                MetaVal::Seq(flat)
            },
            &Self::Dedup => MetaVal::Seq(field_stream.dedup().collect()),
            &Self::Unique => MetaVal::Seq(field_stream.unique().collect()),
            &Self::AllEqual => {
                MetaVal::Bul(
                    match field_stream.next() {
                        None => true,
                        Some(first_val) => {
                            while let Some(next_val) = field_stream.next() {
                                if first_val != next_val {
                                    return MetaVal::Bul(false);
                                }
                            }

                            true
                        }
                    }
                )
            }
        }
    }
}
