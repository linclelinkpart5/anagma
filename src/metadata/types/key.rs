use util::GenConverter;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone)]
pub enum MetaKey {
    // NOTE: Definition order is important, the default impl of Ord ranks Nil before Str(_), which is desired.
    Nil,
    Str(String),
}

impl<S> From<Option<S>> for MetaKey
where
    S: Into<String>
{
    fn from(opt_str: Option<S>) -> Self {
        match opt_str {
            Some(s) => MetaKey::Str(s.into()),
            None => MetaKey::Nil,
        }
    }
}

impl MetaKey {
    pub fn iter_over<'a>(&'a self) -> impl Iterator<Item = &'a String> {
        let closure = move || {
            match *self {
                MetaKey::Nil => {},
                MetaKey::Str(ref s) => { yield s; },
            }
        };

        GenConverter::gen_to_iter(closure)
    }
}

/// A sequence of `MetaKey`s that index into a `MetaBlock`.
/// This works like a hierarchical index on the contents of the `MetaBlock`.
/// An empty `MetaKeyPath` references the entire target `MetaBlock`.
pub type MetaKeyPath = Vec<MetaKey>;

#[cfg(test)]
mod tests {
    use super::MetaKey;

    #[test]
    fn test_iter_over() {
        let inputs_and_expected = vec![
            (MetaKey::Nil, vec![]),
            (MetaKey::Str("sample".to_string()), vec!["sample"]),
        ];

        for (input, expected) in inputs_and_expected {
            let produced: Vec<_> = input.iter_over().collect();
            assert_eq!(expected, produced);
        }
    }
}
