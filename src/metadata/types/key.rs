use std::borrow::Cow;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum MetaKey<'k> {
    Str(Cow<'k, str>)
}

impl<'k> From<String> for MetaKey<'k> {
    fn from(s: String) -> Self {
        Self::Str(Cow::Owned(s))
    }
}

impl<'k> From<&'k str> for MetaKey<'k> {
    fn from(s: &'k str) -> Self {
        Self::Str(Cow::Borrowed(s))
    }
}

impl<'k> std::fmt::Display for MetaKey<'k> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::Str(ref s) => s.fmt(f),
        }
    }
}

#[derive(Clone, Debug)]
pub struct MetaKeyPath<'k>(Cow<'k, [MetaKey<'k>]>);

impl<'k> MetaKeyPath<'k> {
    pub fn new() -> Self {
        Self(Cow::Borrowed(&[]))
    }
}

impl<'k> IntoIterator for MetaKeyPath<'k> {
    type Item = &'k MetaKey<'k>;
    type IntoIter = std::slice::Iter<'k, MetaKey<'k>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'k> IntoIterator for &'k MetaKeyPath<'k> {
    type Item = &'k MetaKey<'k>;
    type IntoIter = std::slice::Iter<'k, MetaKey<'k>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

// impl<'il> IntoIterator for IterableLike<'il> {
//     type Item = Result<MetaVal<'il>, Error>;
//     type IntoIter = IterLike<'il>;

//     fn into_iter(self) -> Self::IntoIter {
//         match self {
//             Self::Stream(s) => IterLike::Stream(s),
//             Self::Sequence(s) => IterLike::Sequence(s.into_iter()),
//         }
//     }
// }

impl<'k> From<MetaKey<'k>> for MetaKeyPath<'k> {
    fn from(mk: MetaKey<'k>) -> Self {
        Self(Cow::Borrowed(&[mk]))
    }
}

impl<'k> From<Vec<MetaKey<'k>>> for MetaKeyPath<'k> {
    fn from(mks: Vec<MetaKey<'k>>) -> Self {
        Self(mks.into())
    }
}

impl<'k> From<&'k [MetaKey<'k>]> for MetaKeyPath<'k> {
    fn from(mks: &'k [MetaKey<'k>]) -> Self {
        Self(mks.into())
    }
}

impl<'k> From<String> for MetaKeyPath<'k> {
    fn from(s: String) -> Self {
        let mk: MetaKey<'k> = s.into();
        mk.into()
    }
}

impl<'k> From<&'k str> for MetaKeyPath<'k> {
    fn from(s: &'k str) -> Self {
        let mk: MetaKey<'k> = s.into();
        mk.into()
    }
}

// impl<'k, SS, S> From<SS> for MetaKeyPath<'k>
// where
//     SS: Into<Cow<'k, [S]>>,
//     S: Into<Cow<'k, str>>,
// {
//     fn from(ss: SS) -> Self {
//         let mk: MetaKey = ss.into();
//         mk.into()
//     }
// }

// impl<'k> From<Vec<String>> for MetaKeyPath<'k> {
//     fn from(ss: Vec<String>) -> Self {
//         let mut mks = vec![];

//         for s in ss {
//             let mk: MetaKey = s.into();
//             mks.push(Cow::Owned(mk));
//         }

//         mks.into()
//     }
// }

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
