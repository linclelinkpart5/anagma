use serde::Deserialize;
use strum::{EnumString, EnumIter, AsRefStr};

/// Represents all the different metadata formats that are supported.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Deserialize, EnumString, EnumIter, AsRefStr)]
#[strum(serialize_all = "snake_case")]
pub enum Format {
    #[strum(serialize = "JSON", serialize = "json")]
    Json,
    #[strum(serialize = "YML", serialize = "yml")]
    Yaml,
}
