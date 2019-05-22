#[derive(Clone, Copy, Debug)]
pub enum IterAdaptor {
    Flatten,
    Dedup,
    Unique,
}
