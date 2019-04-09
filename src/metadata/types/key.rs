use std::borrow::Cow;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum MetaKey{
    Str(String)
}

impl From<String> for MetaKey {
    fn from(s: String) -> Self {
        Self::Str(s)
    }
}

impl From<&str> for MetaKey {
    fn from(s: &str) -> Self {
        Self::Str(s.into())
    }
}

impl std::fmt::Display for MetaKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::Str(ref s) => s.fmt(f),
        }
    }
}

pub struct MetaKeyPath<'mk>(Vec<Cow<'mk, MetaKey>>);

impl<'mk> From<&'mk MetaKey> for MetaKeyPath<'mk> {
    fn from(mk: &'mk MetaKey) -> Self {
        Self(vec![Cow::Borrowed(mk)])
    }
}

impl<'mk> From<MetaKey> for MetaKeyPath<'mk> {
    fn from(mk: MetaKey) -> Self {
        Self(vec![Cow::Owned(mk)])
    }
}

impl<'mk> From<Vec<&'mk MetaKey>> for MetaKeyPath<'mk> {
    fn from(mks: Vec<&'mk MetaKey>) -> Self {
        Self(mks.into_iter().map(Cow::Borrowed).collect())
    }
}

impl<'mk> From<Vec<MetaKey>> for MetaKeyPath<'mk> {
    fn from(mks: Vec<MetaKey>) -> Self {
        Self(mks.into_iter().map(Cow::Owned).collect())
    }
}

impl<'mk> From<String> for MetaKeyPath<'mk> {
    fn from(s: String) -> Self {
        let mk: MetaKey = s.into();
        mk.into()
    }
}

impl<'mk> From<Vec<String>> for MetaKeyPath<'mk> {
    fn from(ss: Vec<String>) -> Self {
        let mut mks = vec![];

        for s in ss {
            let mk: MetaKey = s.into();
            mks.push(Cow::Owned(mk));
        }

        Self(mks)
    }
}

#[cfg(test)]
mod tests {
    use super::MetaKey;

    #[test]
    fn test_deserialize() {
        let expected = MetaKey::from("key_a");

        let input = r#""key_a""#;
        let produced = serde_json::from_str::<MetaKey>(&input).unwrap();
        assert_eq!(expected, produced);

        let input = "key_a";
        let produced = serde_yaml::from_str::<MetaKey>(&input).unwrap();
        assert_eq!(expected, produced);
    }
}
