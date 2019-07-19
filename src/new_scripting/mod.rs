pub mod token;
pub mod operators;
pub(crate) mod operator_impl;
pub mod generic_iterable;

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(test, derive(EnumDiscriminants))]
#[cfg_attr(test, strum_discriminants(name(ErrorKind)))]
pub enum Error {
    Operator,
    #[cfg(test)] Sentinel,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::Operator => write!(f, "operator error"),
            #[cfg(test)] Self::Sentinel => write!(f, "sentinel error, only for testing"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
