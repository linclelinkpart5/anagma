use super::Predicate;

#[derive(Clone, Copy, Debug)]
pub enum Converter {
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
    Predicate(Predicate),
}

impl Converter {
}

#[cfg(test)]
mod tests {
}
