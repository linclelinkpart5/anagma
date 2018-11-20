mod inherit;
mod collect;

use self::inherit::InheritMethod;
use self::collect::CollectMethod;

/// Different ways to process parent metadata into desired outputs.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FallbackMethod {
    Inherit(InheritMethod),
    Collect(CollectMethod),
}
