//! # Bel
//! 
//! Generate TypeScript interfaces from Rust structs/traits - useful for JSON RPC
//! 
//! This is a Rust port of the original Go implementation.

pub mod extract;
pub mod generator;
pub mod typescript;
pub mod enums;

pub use extract::*;
pub use generator::*;
pub use typescript::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
