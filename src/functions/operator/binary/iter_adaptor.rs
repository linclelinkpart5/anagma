use crate::functions::Error;
use crate::functions::util::StreamAdaptor;

#[derive(Clone, Copy, Debug)]
pub enum IterAdaptor {
}

impl IterAdaptor {
    pub fn process<'sa>(&self, sa: StreamAdaptor<'sa>) -> Result<StreamAdaptor<'sa>, Error> {
        Ok(sa)
    }
}
