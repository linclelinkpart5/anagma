use crate::functions::Error;
use crate::functions::util::StreamAdaptor;

#[derive(Clone, Copy, Debug)]
pub enum IterAdaptor {
    StepBy,
    Chain,
    Zip,
    Map,
    Filter,
    SkipWhile,
    TakeWhile,
    Skip,
    Take,
    Interleave,
    Intersperse,
    Chunks,
    Windows,
}

impl IterAdaptor {
    pub fn process<'sa>(&self, sa: StreamAdaptor<'sa>) -> Result<StreamAdaptor<'sa>, Error> {
        Ok(sa)
    }
}
