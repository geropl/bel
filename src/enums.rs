//! Enum handling and extraction utilities

use std::collections::HashMap;
use syn::{ItemEnum, Variant, Expr, Lit};

/// Enum value representation
#[derive(Debug, Clone)]
pub enum EnumValue {
    String(String),
    Number(i64),
    Identifier(String),
}

impl EnumValue {
    pub fn to_typescript(&self) -> String {
        match self {
            EnumValue::String(s) => format!("\"{}\"", s),
            EnumValue::Number(n) => n.to_string(),
            EnumValue::Identifier(i) => i.clone(),
        }
    }
}

/// Extracted enum information
#[derive(Debug, Clone)]
pub struct ExtractedEnum {
    pub name: String,
    pub variants: Vec<(String, Option<EnumValue>)>,
    pub doc: Option<String>,
}

/// Extract enum information from Rust enum
pub fn extract_enum(item: &ItemEnum) -> Result<ExtractedEnum, Box<dyn std::error::Error>> {
    let name = item.ident.to_string();
    let mut variants = Vec::new();

    for variant in &item.variants {
        let variant_name = variant.ident.to_string();
        let value = extract_variant_value(variant)?;
        variants.push((variant_name, value));
    }

    Ok(ExtractedEnum {
        name,
        variants,
        doc: extract_doc(&item.attrs),
    })
}

fn extract_variant_value(variant: &Variant) -> Result<Option<EnumValue>, Box<dyn std::error::Error>> {
    if let Some((_, expr)) = &variant.discriminant {
        match expr {
            Expr::Lit(expr_lit) => {
                match &expr_lit.lit {
                    Lit::Str(lit_str) => Ok(Some(EnumValue::String(lit_str.value()))),
                    Lit::Int(lit_int) => {
                        let value = lit_int.base10_parse::<i64>()?;
                        Ok(Some(EnumValue::Number(value)))
                    }
                    _ => Ok(None),
                }
            }
            Expr::Path(expr_path) => {
                if let Some(ident) = expr_path.path.get_ident() {
                    Ok(Some(EnumValue::Identifier(ident.to_string())))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    } else {
        Ok(None)
    }
}

fn extract_doc(attrs: &[syn::Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("doc") {
            if let Ok(meta) = attr.meta.require_name_value() {
                if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(lit_str), .. }) = &meta.value {
                    return Some(lit_str.value().trim().to_string());
                }
            }
        }
    }
    None
}

/// Generate TypeScript enum code
pub fn generate_typescript_enum(extracted_enum: &ExtractedEnum, as_sum_type: bool) -> String {
    let mut result = String::new();

    // Add documentation if present
    if let Some(doc) = &extracted_enum.doc {
        result.push_str(&format!("/**\n * {}\n */\n", doc));
    }

    if as_sum_type {
        // Generate as sum type
        result.push_str(&format!("export type {} =", extracted_enum.name));
        for (i, (variant_name, value)) in extracted_enum.variants.iter().enumerate() {
            if i == 0 {
                result.push('\n');
                result.push_str("    ");
            } else {
                result.push_str(" |\n    ");
            }
            
            if let Some(val) = value {
                result.push_str(&val.to_typescript());
            } else {
                result.push_str(&format!("\"{}\"", variant_name));
            }
        }
        result.push_str(";\n");
    } else {
        // Generate as enum
        result.push_str(&format!("export enum {} {{\n", extracted_enum.name));
        for (i, (variant_name, value)) in extracted_enum.variants.iter().enumerate() {
            result.push_str(&format!("    {}", variant_name));
            if let Some(val) = value {
                result.push_str(&format!(" = {}", val.to_typescript()));
            } else {
                result.push_str(&format!(" = {}", i));
            }
            result.push_str(",\n");
        }
        result.push_str("}\n");
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_extract_simple_enum() {
        let item: ItemEnum = parse_quote! {
            enum Status {
                Active,
                Inactive,
            }
        };

        let extracted = extract_enum(&item).unwrap();
        assert_eq!(extracted.name, "Status");
        assert_eq!(extracted.variants.len(), 2);
        assert_eq!(extracted.variants[0].0, "Active");
        assert_eq!(extracted.variants[1].0, "Inactive");
    }

    #[test]
    fn test_extract_enum_with_values() {
        let item: ItemEnum = parse_quote! {
            enum HttpStatus {
                Ok = 200,
                NotFound = 404,
            }
        };

        let extracted = extract_enum(&item).unwrap();
        assert_eq!(extracted.name, "HttpStatus");
        assert_eq!(extracted.variants.len(), 2);
        
        if let Some(EnumValue::Number(200)) = &extracted.variants[0].1 {
            // OK
        } else {
            panic!("Expected number value 200");
        }
    }

    #[test]
    fn test_generate_typescript_enum() {
        let extracted = ExtractedEnum {
            name: "Status".to_string(),
            variants: vec![
                ("Active".to_string(), None),
                ("Inactive".to_string(), None),
            ],
            doc: None,
        };

        let result = generate_typescript_enum(&extracted, false);
        assert!(result.contains("export enum Status"));
        assert!(result.contains("Active = 0"));
        assert!(result.contains("Inactive = 1"));
    }

    #[test]
    fn test_generate_typescript_sum_type() {
        let extracted = ExtractedEnum {
            name: "Status".to_string(),
            variants: vec![
                ("Active".to_string(), None),
                ("Inactive".to_string(), None),
            ],
            doc: None,
        };

        let result = generate_typescript_enum(&extracted, true);
        assert!(result.contains("export type Status ="));
        assert!(result.contains("\"Active\""));
        assert!(result.contains("\"Inactive\""));
    }
}
