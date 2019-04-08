#[derive(Clone, Copy, Debug)]
pub enum BinaryOp {
    // (Iterable<V>, Usize) -> V
    Nth,
    // (Stream<V>, Usize) -> Stream<V>
    // (Sequence<V>, Usize) -> Sequence<V>
    StepBy,
    // (Sequence<V>, Sequence<V>) -> Sequence<V>
    // (Stream<V>, Iterable<V>) -> Stream<V>
    // (Iterable<V>, Stream<V>) -> Stream<V>
    Chain,
    // (Sequence<V>, Sequence<V>) -> Sequence<Sequence<V>>
    // (Stream<V>, Iterable<V>) -> Stream<Sequence<V>>
    // (Iterable<V>, Stream<V>) -> Stream<Sequence<V>>
    Zip,
    // (Stream<V>, UnaryOp) -> Stream<V>
    // (Sequence<V>, UnaryOp) -> Sequence<V>
    Map,
    // (Stream<V>, Predicate) -> Stream<V>
    // (Sequence<V>, Predicate) -> Sequence<V>
    Filter,
    // (Stream<V>, Predicate) -> Stream<V>
    // (Sequence<V>, Predicate) -> Sequence<V>
    SkipWhile,
    // (Stream<V>, Predicate) -> Stream<V>
    // (Sequence<V>, Predicate) -> Sequence<V>
    TakeWhile,
    // (Stream<V>, Usize) -> Stream<V>
    // (Sequence<V>, Usize) -> Sequence<V>
    Skip,
    // (Stream<V>, Usize) -> Stream<V>
    // (Sequence<V>, Usize) -> Sequence<V>
    Take,
    // (Iterable<V>, Predicate) -> Boolean
    All,
    // (Iterable<V>, Predicate) -> Boolean
    Any,
    // (Iterable<V>, Predicate) -> V
    Find,
    // (Iterable<V>, Predicate) -> Usize
    Position,
    // (Sequence<V>, Sequence<V>) -> Sequence<V>
    // (Stream<V>, Iterable<V>) -> Stream<V>
    // (Iterable<V>, Stream<V>) -> Stream<V>
    Interleave,
    // (Stream<V>, V) -> Stream<V>
    // (Sequence<V>, V) -> Sequence<V>
    Intersperse,
    // (Stream<V>, Usize) -> Stream<Sequence<V>>
    // (Sequence<V>, Usize) -> Sequence<Sequence<V>>
    Chunks,
    // (Stream<V>, Usize) -> Stream<Sequence<V>>
    // (Sequence<V>, Usize) -> Sequence<Sequence<V>>
    Windows,
}
