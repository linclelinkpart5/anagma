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
    First,
}

impl FieldConsumer {
    pub fn process(&self, field_producer: &mut SimpleMetaFieldProducer) -> Result<MetaVal, Error> {
        match self {
            &Self::First => Ok(field_producer.next().unwrap_or_else(|| Ok(MetaVal::Nil))?),
        }
    }
}
