use metadata::types::MetaVal;

/// Different ways to process child metadata into desired outputs.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AggMethod {
    Collect,
    First,
}

impl AggMethod {
    pub fn aggregate<II>(&self, mvs: II) -> MetaVal
    where
        II: IntoIterator<Item = MetaVal>,
    {
        let mut mvs = mvs.into_iter();

        match *self {
            AggMethod::First => {
                mvs.next().unwrap_or(MetaVal::Nil)
            },
            AggMethod::Collect => {
                MetaVal::Seq(mvs.collect())
            },
        }
    }
}
