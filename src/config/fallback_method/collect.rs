use metadata::types::MetaVal;

/// Different ways to process child metadata into desired outputs.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CollectMethod {
    Iterate,
    First,
}

impl CollectMethod {
    pub fn process<II>(self, mvs: II) -> MetaVal
    where
        II: IntoIterator<Item = MetaVal>,
    {
        let mut mvs = mvs.into_iter();

        match self {
            CollectMethod::First => {
                mvs.next().unwrap_or(MetaVal::Nil)
            },
            CollectMethod::Iterate => {
                MetaVal::Seq(mvs.collect())
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use metadata::types::MetaVal;

    use super::CollectMethod;

    #[test]
    fn test_process() {
        let inputs_and_expected = vec![
            (
                (
                    CollectMethod::First,
                    vec![
                        MetaVal::Str(String::from("A")),
                    ],
                ),
                MetaVal::Str(String::from("A")),
            ),
            (
                (
                    CollectMethod::First,
                    vec![
                        MetaVal::Str(String::from("A")),
                        MetaVal::Str(String::from("B")),
                        MetaVal::Str(String::from("C")),
                    ],
                ),
                MetaVal::Str(String::from("A")),
            ),
            (
                (
                    CollectMethod::First,
                    vec![],
                ),
                MetaVal::Nil,
            ),
            (
                (
                    CollectMethod::Iterate,
                    vec![
                        MetaVal::Str(String::from("A")),
                        MetaVal::Str(String::from("B")),
                        MetaVal::Str(String::from("C")),
                    ],
                ),
                MetaVal::Seq(
                    vec![
                        MetaVal::Str(String::from("A")),
                        MetaVal::Str(String::from("B")),
                        MetaVal::Str(String::from("C")),
                    ]
                ),
            ),
            (
                (
                    CollectMethod::Iterate,
                    vec![],
                ),
                MetaVal::Seq(vec![]),
            ),
        ];

        for (input, expected) in inputs_and_expected {
            let (collect_method, mvs) = input;

            let produced = collect_method.process(mvs);
            assert_eq!(expected, produced);
        }
    }
}
