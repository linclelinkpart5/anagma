use metadata::types::MetaVal;
use metadata::streams::value::SimpleMetaValueStream;
use metadata::streams::value::Error as ValueStreamError;

use itertools::Itertools;

#[derive(Debug)]
pub enum Error {
    FieldProducer(ValueStreamError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::FieldProducer(ref err) => write!(f, "field producer error: {}", err),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Self::FieldProducer(ref err) => Some(err),
        }
    }
}

impl From<ValueStreamError> for Error {
    fn from(err: ValueStreamError) -> Self {
        Self::FieldProducer(err)
    }
}

/// Methods for processing and fetching useful data from field producers.
/// Unless specified, all methods only operate on valid values, and ignore errors.
pub enum FieldConsumer {
    /// Return all values from the producer.
    Collect,
    /// Count the number of values from the producer.
    Count,
    /// Return the first found value from the producer.
    First,
    /// Return the last found value from the producer.
    Last,
    /// Flattens any sequence values from the producer, leaving other meta values unchanged.
    Flatten,
    /// Filters out duplicates from consecutive runs of values.
    Dedup,
    /// Filters out values that have already been produced once.
    Unique,
    /// Equals true if all the values in the producer are equal to each other.
    AllEqual,
}

impl FieldConsumer {
    pub fn process(&self, field_producer: &mut SimpleMetaValueStream) -> MetaVal {
        match self {
            &Self::Collect => MetaVal::Seq(field_producer.collect()),
            &Self::Count => MetaVal::Int(field_producer.count() as i64),
            &Self::First => field_producer.next().unwrap_or_else(|| MetaVal::Nil),
            &Self::Last => field_producer.last().unwrap_or_else(|| MetaVal::Nil),
            &Self::Flatten => {
                let mut flat = vec![];

                for mv in field_producer {
                    match mv {
                        MetaVal::Seq(seq) => flat.extend(seq.into_iter()),
                        o => flat.push(o),
                    }
                }

                MetaVal::Seq(flat)
            },
            &Self::Dedup => MetaVal::Seq(field_producer.dedup().collect()),
            &Self::Unique => MetaVal::Seq(field_producer.unique().collect()),
            &Self::AllEqual => {
                MetaVal::Bul(
                    match field_producer.next() {
                        None => true,
                        Some(first_val) => {
                            while let Some(next_val) = field_producer.next() {
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
