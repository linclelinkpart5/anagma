pub mod file_walkers;
pub mod number;

pub use number::Number;

use std::path::Path;
use std::path::PathBuf;
use std::path::Component;

pub fn _is_valid_item_name<S: AsRef<str>>(s: S) -> bool {
    let s = s.as_ref();
    let s_path = Path::new(s);
    let components: Vec<_> = s_path.components().collect();

    // If an item name does not have exactly one component, it cannot be valid.
    if components.len() != 1 {
        return false;
    }

    // The single component must be normal.
    match components[0] {
        Component::Normal(_) => {},
        _ => { return false; },
    }

    // If recreating the path from the component does not match the original, it cannot be valid.
    let mut p = PathBuf::new();
    for c in components {
        p.push(c.as_os_str());
    }

    p.as_os_str() == s_path.as_os_str()
}
