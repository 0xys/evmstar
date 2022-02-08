pub mod model;
pub mod resume;
pub mod interpreter;
pub mod executor;
pub mod utils;
pub mod host;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
