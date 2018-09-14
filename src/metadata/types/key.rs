use util::GenConverter;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone)]
pub enum MetaKey {
    Nil,
    Str(String),
}

impl MetaKey {
    pub fn iter_over<'a>(&'a self) -> impl Iterator<Item = &'a String> {
        let closure = move || {
            match *self {
                MetaKey::Nil => {},
                MetaKey::Str(ref s) => { yield s; },
            }
        };

        GenConverter::gen_to_iter(closure)
    }
}
