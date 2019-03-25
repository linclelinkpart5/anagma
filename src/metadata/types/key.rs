#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum MetaKey{
    Str(String)
}

impl<S> From<S> for MetaKey
where
    S: Into<String>
{
    fn from(s: S) -> Self {
        Self::Str(s.into())
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
