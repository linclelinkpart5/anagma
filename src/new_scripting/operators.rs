#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[derive(EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum UnaryOp {
    // Iterable -> Sequence
    Collect,
    Rev,
    Sort,

    // Iterable -> Usize
    Count,

    // Iterable<T> -> T
    First,
    Last,

    // Iterable<Number> -> Number
    MinIn,
    MaxIn,
    Sum,
    Prod,

    // Iterable -> Boolean
    AllEqual,

    // Sequence -> Sequence
    // Producer -> Producer
    Flatten,
    Dedup,
    Unique,

    // Number -> Number
    Neg,
    Abs,

    // Boolean -> Boolean
    Not,

    // T -> T
    Noop,

    // Mapping<K, _> -> Sequence<K>
    Keys,

    // Mapping<_, V> -> Sequence<V>
    Values,

    // Mapping<_, V> -> V
    Pick,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[derive(EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum BinaryOp {
    // Iterable<T>, Usize -> T
    Nth,

    // Iterable, Predicate -> Boolean
    All,
    Any,

    // Iterable<T>, Predicate -> T
    Find,

    // Iterable, Predicate -> Usize
    Position,

    // Sequence, Predicate -> Sequence
    // Producer, Predicate -> Producer
    Filter,
    Map,
    SkipWhile,
    TakeWhile,

    // Sequence, Usize -> Sequence
    // Producer, Usize -> Producer
    Skip,
    Take,
    StepBy,

    // Sequence, Sequence -> Sequence
    // Iterable, Iterable -> Producer
    Chain,
    Zip,

    // Boolean, Expression -> Boolean
    And,
    Or,

    // Boolean, Boolean -> Boolean
    Xor,

    // Number, Number -> Boolean
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,

    // Number, Number -> Number
    Add,
    Sub,
    Mul,
    Div,
    Rem,

    // Mapping<K, V>, K -> V
    Lookup,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[derive(EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum TernaryOp {
    // Boolean, Expression<Any>, Expression<Any> -> Any
    If,

    // Sequence, Usize, Any -> Sequence
    // Producer, Usize, Any -> Producer
    Pad,
}
