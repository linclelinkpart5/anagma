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

pub struct MetaKeyPath<'k>(Cow<'k, [MetaKey<'k>]>);

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

impl<'k> From<&[MetaKey<'k>]> for MetaKeyPath<'k> {
    fn from(mks: &[MetaKey<'k>]) -> Self {
        Self(mks.into())
    }
}

impl<'k> From<String> for MetaKeyPath<'k> {
    fn from(s: String) -> Self {
        let mk: MetaKey = s.into();
        mk.into()
    }
}

impl<'k> From<&str> for MetaKeyPath<'k> {
    fn from(s: &str) -> Self {
        let mk: MetaKey = s.into();
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

impl<'k> From<Vec<String>> for MetaKeyPath<'k> {
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
