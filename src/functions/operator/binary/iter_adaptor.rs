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
