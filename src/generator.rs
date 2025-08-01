//! Generate TypeScript code from extracted type information

use std::collections::HashMap;
use std::io::Write;
use crate::extract::{TypescriptType, TypescriptField, TypescriptMethod};

/// Generator options
#[derive(Debug, Clone)]
pub struct GeneratorOptions {
    pub namespace: Option<String>,
    pub preamble: Option<String>,
    pub generate_enums_as_sum_types: bool,
    pub sort_alphabetically: bool,
}

impl Default for GeneratorOptions {
    fn default() -> Self {
        Self {
            namespace: None,
            preamble: Some("// generated using bel\n// DO NOT MODIFY".to_string()),
            generate_enums_as_sum_types: false,
            sort_alphabetically: false,
        }
    }
}

/// TypeScript code generator
pub struct Generator {
    options: GeneratorOptions,
}

impl Generator {
    pub fn new(options: GeneratorOptions) -> Self {
        Self { options }
    }

    /// Generate TypeScript code from extracted types
    pub fn generate<W: Write>(&self, types: &HashMap<String, TypescriptType>, writer: &mut W) -> Result<(), Box<dyn std::error::Error>> {
        // Write preamble
        if let Some(preamble) = &self.options.preamble {
            writeln!(writer, "{}", preamble)?;
        }

        // Start namespace if specified
        if let Some(namespace) = &self.options.namespace {
            writeln!(writer, "export namespace {} {{", namespace)?;
        }

        // Sort types if requested
        let mut type_names: Vec<_> = types.keys().collect();
        if self.options.sort_alphabetically {
            type_names.sort();
        }

        // Generate each type
        for type_name in type_names {
            if let Some(ts_type) = types.get(type_name) {
                self.generate_type(ts_type, writer)?;
                writeln!(writer)?;
            }
        }

        // End namespace if specified
        if self.options.namespace.is_some() {
            writeln!(writer, " }}")?;
        }

        Ok(())
    }

    fn generate_type<W: Write>(&self, ts_type: &TypescriptType, writer: &mut W) -> Result<(), Box<dyn std::error::Error>> {
        match ts_type {
            TypescriptType::Interface { name, fields, doc } => {
                self.write_doc(doc, writer)?;
                writeln!(writer, "export interface {} {{", name)?;
                
                let mut field_names: Vec<_> = fields.iter().collect();
                if self.options.sort_alphabetically {
                    field_names.sort_by(|a, b| a.name.cmp(&b.name));
                }

                for field in field_names {
                    self.generate_field(field, writer)?;
                }
                writeln!(writer, "}}")?;
            }
            TypescriptType::Enum { name, variants, doc } => {
                self.write_doc(doc, writer)?;
                if self.options.generate_enums_as_sum_types {
                    self.generate_sum_type_enum(name, variants, writer)?;
                } else {
                    self.generate_enum(name, variants, writer)?;
                }
            }
            TypescriptType::Trait { name, methods, doc } => {
                self.write_doc(doc, writer)?;
                writeln!(writer, "export interface {} {{", name)?;
                
                let mut method_names: Vec<_> = methods.iter().collect();
                if self.options.sort_alphabetically {
                    method_names.sort_by(|a, b| a.name.cmp(&b.name));
                }

                for method in method_names {
                    self.generate_method(method, writer)?;
                }
                writeln!(writer, "}}")?;
            }
        }
        Ok(())
    }

    fn generate_field<W: Write>(&self, field: &TypescriptField, writer: &mut W) -> Result<(), Box<dyn std::error::Error>> {
        self.write_doc(&field.doc, writer)?;
        let optional = if field.optional { "?" } else { "" };
        writeln!(writer, "    {}{}: {}", field.name, optional, field.ts_type)?;
        Ok(())
    }

    fn generate_method<W: Write>(&self, method: &TypescriptMethod, writer: &mut W) -> Result<(), Box<dyn std::error::Error>> {
        self.write_doc(&method.doc, writer)?;
        let params: Vec<String> = method.params.iter()
            .map(|p| format!("{}: {}", p.name, p.ts_type))
            .collect();
        writeln!(writer, "    {}({}): {}", method.name, params.join(", "), method.return_type)?;
        Ok(())
    }

    fn generate_enum<W: Write>(&self, name: &str, variants: &[String], writer: &mut W) -> Result<(), Box<dyn std::error::Error>> {
        writeln!(writer, "export enum {} {{", name)?;
        for (i, variant) in variants.iter().enumerate() {
            writeln!(writer, "    {} = {},", variant, i)?;
        }
        writeln!(writer, "}}")?;
        Ok(())
    }

    fn generate_sum_type_enum<W: Write>(&self, name: &str, variants: &[String], writer: &mut W) -> Result<(), Box<dyn std::error::Error>> {
        write!(writer, "export type {} =", name)?;
        for (i, variant) in variants.iter().enumerate() {
            if i == 0 {
                writeln!(writer)?;
                write!(writer, "    \"{}\"", variant)?;
            } else {
                writeln!(writer, " |")?;
                write!(writer, "    \"{}\"", variant)?;
            }
        }
        writeln!(writer, ";")?;
        Ok(())
    }

    fn write_doc<W: Write>(&self, doc: &Option<String>, writer: &mut W) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(doc_text) = doc {
            writeln!(writer, "/**")?;
            writeln!(writer, " * {}", doc_text)?;
            writeln!(writer, " */")?;
        }
        Ok(())
    }
}

/// Convenience function to generate TypeScript code
pub fn generate<W: Write>(types: &HashMap<String, TypescriptType>, options: GeneratorOptions, writer: &mut W) -> Result<(), Box<dyn std::error::Error>> {
    let generator = Generator::new(options);
    generator.generate(types, writer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extract::{TypescriptField, TypescriptType};
    use std::collections::HashMap;

    #[test]
    fn test_generate_interface() {
        let mut types = HashMap::new();
        types.insert("Demo".to_string(), TypescriptType::Interface {
            name: "Demo".to_string(),
            fields: vec![
                TypescriptField {
                    name: "foo".to_string(),
                    ts_type: "string".to_string(),
                    optional: true,
                    doc: None,
                },
                TypescriptField {
                    name: "bar".to_string(),
                    ts_type: "number".to_string(),
                    optional: false,
                    doc: None,
                },
            ],
            doc: None,
        });

        let mut output = Vec::new();
        generate(&types, GeneratorOptions::default(), &mut output).unwrap();
        let result = String::from_utf8(output).unwrap();
        
        assert!(result.contains("export interface Demo"));
        assert!(result.contains("foo?: string"));
        assert!(result.contains("bar: number"));
    }

    #[test]
    fn test_generate_enum() {
        let mut types = HashMap::new();
        types.insert("Status".to_string(), TypescriptType::Enum {
            name: "Status".to_string(),
            variants: vec!["Active".to_string(), "Inactive".to_string()],
            doc: None,
        });

        let mut output = Vec::new();
        generate(&types, GeneratorOptions::default(), &mut output).unwrap();
        let result = String::from_utf8(output).unwrap();
        
        assert!(result.contains("export enum Status"));
        assert!(result.contains("Active = 0"));
        assert!(result.contains("Inactive = 1"));
    }
}
