use std::borrow::Cow;

use crate::metadata::types::MetaVal;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Generic,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::Generic => write!(f, "generic error"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn collect<'a, I>(it: impl Iterator<Item = Result<I>>) -> Result<Vec<MetaVal>>
where
    I: Into<Cow<'a, MetaVal>>,
{
    let mut ret = vec![];
    for res in it {
        let mv = res?.into().into_owned();
        ret.push(mv);
    }

    Ok(ret)
}
