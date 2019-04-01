use metadata::types::MetaVal;
use metadata::producer::field::SimpleMetaFieldProducer;
use metadata::producer::field::Error as FieldProducerError;

#[derive(Debug)]
pub enum Error {
    FieldProducer(FieldProducerError),
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

impl From<FieldProducerError> for Error {
    fn from(err: FieldProducerError) -> Self {
        Self::FieldProducer(err)
    }
}

pub enum FieldConsumer {
    /// Return the first found value from the producer.
    First,
    /// Return the last found value from the producer.
    Last,
    /// Return all values from the producer.
    Collect,
}

impl FieldConsumer {
    pub fn process(&self, field_producer: &mut SimpleMetaFieldProducer) -> MetaVal {
        match self {
            &Self::First => field_producer.next().unwrap_or_else(|| MetaVal::Nil),
            &Self::Last => field_producer.last().unwrap_or_else(|| MetaVal::Nil),
            &Self::Collect => MetaVal::Seq(field_producer.collect()),
        }
    }
}
