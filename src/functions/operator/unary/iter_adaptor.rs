use crate::functions::Error;
use crate::functions::util::StreamAdaptor;
use crate::functions::util::FlattenAdaptor;
use crate::functions::util::DedupAdaptor;
use crate::functions::util::UniqueAdaptor;

#[derive(Clone, Copy, Debug)]
pub enum IterAdaptor {
    Flatten,
    Dedup,
    Unique,
}

impl IterAdaptor {
    pub fn process<'sa>(&self, sa: StreamAdaptor<'sa>) -> Result<StreamAdaptor<'sa>, Error> {
        Ok(match self {
            &Self::Flatten => StreamAdaptor::Flatten(FlattenAdaptor::new(sa)),
            &Self::Dedup => StreamAdaptor::Dedup(DedupAdaptor::new(sa)),
            &Self::Unique => StreamAdaptor::Unique(UniqueAdaptor::new(sa)),
        })
    }
}
