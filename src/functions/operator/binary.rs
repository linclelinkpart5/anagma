pub mod converter;
pub mod predicate;
pub mod iter_consumer;
pub mod iter_adaptor;
pub mod imp;

pub use self::converter::Converter;
pub use self::predicate::Predicate;
pub use self::iter_consumer::IterConsumer;
pub use self::iter_adaptor::IterAdaptor;

#[derive(Clone, Copy, Debug)]
pub enum Op {
    Nth,
    StepBy,
    Chain,
    Zip,
    Map,
    Filter,
    SkipWhile,
    TakeWhile,
    Skip,
    Take,
    All,
    Any,
    Find,
    Position,
    Interleave,
    Intersperse,
    Chunks,
    Windows,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}
