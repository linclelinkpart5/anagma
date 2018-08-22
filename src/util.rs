use std::ops::Generator;
use std::ops::GeneratorState;

pub struct GenConverter;

impl GenConverter {
    pub fn gen_to_iter<G>(g: G) -> impl Iterator<Item = G::Yield>
    where G: Generator<Return = ()> {
        struct It<G>(G);

        impl<G: Generator<Return = ()>> Iterator for It<G> {
            type Item = G::Yield;

            fn next(&mut self) -> Option<Self::Item> {
                unsafe {
                    match self.0.resume() {
                        GeneratorState::Yielded(y) => Some(y),
                        GeneratorState::Complete(()) => None,
                    }
                }
            }
        }

        It(g)
    }
}
