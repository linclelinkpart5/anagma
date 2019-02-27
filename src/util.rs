use std::ops::Generator;
use std::ops::GeneratorState;
use std::path::Path;
use std::path::PathBuf;
use std::path::Component;
use std::pin::Pin;

pub struct GenConverter;

impl GenConverter {
    pub fn gen_to_iter<G>(g: G) -> impl Iterator<Item = G::Yield>
    where G: Generator<Return = ()> + std::marker::Unpin {
        struct It<G>(G);

        impl<G: Generator<Return = ()> + std::marker::Unpin> Iterator for It<G> {
            type Item = G::Yield;

            fn next(&mut self) -> Option<Self::Item> {
                match Pin::new(&mut self.0).resume() {
                    GeneratorState::Yielded(y) => Some(y),
                    GeneratorState::Complete(()) => None,
                }
            }
        }

        It(g)
    }
}

pub fn is_valid_item_name<S: AsRef<str>>(s: S) -> bool {
    let s = s.as_ref();
    let s_path = Path::new(s);
    let components: Vec<_> = s_path.components().collect();

    // If an item name does not have exactly one component, it cannot be valid.
    if components.len() != 1 {
        return false;
    }

    // The single component but be normal.
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
