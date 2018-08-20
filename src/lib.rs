#[macro_use] extern crate failure;
extern crate regex;

mod library;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
