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

/// Main API for extracting and generating TypeScript from Rust code
pub fn extract_and_generate(
    source: &str,
    extract_options: extract::ExtractOptions,
    generator_options: generator::GeneratorOptions,
) -> Result<String, Box<dyn std::error::Error>> {
    let types = extract::extract(source, extract_options)?;
    let mut output = Vec::new();
    generator::generate(&types, generator_options, &mut output)?;
    Ok(String::from_utf8(output)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_pipeline() {
        let source = r#"
            /// A demo struct for testing
            pub struct Demo {
                /// The foo field
                pub foo: String,
                pub bar: u32,
            }
        "#;

        let result = extract_and_generate(
            source,
            extract::ExtractOptions::default(),
            generator::GeneratorOptions::default(),
        ).unwrap();

        assert!(result.contains("export interface Demo"));
        assert!(result.contains("foo: string"));
        assert!(result.contains("bar: number"));
    }

    #[test]
    fn test_enum_generation() {
        let source = r#"
            pub enum Status {
                Active,
                Inactive,
            }
        "#;

        let result = extract_and_generate(
            source,
            extract::ExtractOptions::default(),
            generator::GeneratorOptions::default(),
        ).unwrap();

        assert!(result.contains("export enum Status"));
        assert!(result.contains("Active = 0"));
        assert!(result.contains("Inactive = 1"));
    }

    #[test]
    fn test_trait_generation() {
        let source = r#"
            pub trait DemoService {
                fn say_hello(&self, name: String, msg: String) -> String;
            }
        "#;

        let result = extract_and_generate(
            source,
            extract::ExtractOptions::default(),
            generator::GeneratorOptions::default(),
        ).unwrap();

        assert!(result.contains("export interface DemoService"));
        assert!(result.contains("say_hello"));
    }
}
