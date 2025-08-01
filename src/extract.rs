//! Extract TypeScript type information from Rust types

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use syn::{Type, Field, ItemStruct, ItemEnum, ItemTrait};

/// Options for the extraction process
#[derive(Debug, Clone)]
pub struct ExtractOptions {
    pub embed_structs: bool,
    pub follow_structs: bool,
    pub no_anon_structs: bool,
    pub sort_alphabetically: bool,
}

impl Default for ExtractOptions {
    fn default() -> Self {
        Self {
            embed_structs: false,
            follow_structs: false,
            no_anon_structs: false,
            sort_alphabetically: false,
        }
    }
}

/// TypeScript type representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TypescriptType {
    Interface {
        name: String,
        fields: Vec<TypescriptField>,
        doc: Option<String>,
    },
    Enum {
        name: String,
        variants: Vec<String>,
        doc: Option<String>,
    },
    Trait {
        name: String,
        methods: Vec<TypescriptMethod>,
        doc: Option<String>,
    },
}

/// TypeScript field representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypescriptField {
    pub name: String,
    pub ts_type: String,
    pub optional: bool,
    pub doc: Option<String>,
}

/// TypeScript method representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypescriptMethod {
    pub name: String,
    pub params: Vec<TypescriptParam>,
    pub return_type: String,
    pub doc: Option<String>,
}

/// TypeScript parameter representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypescriptParam {
    pub name: String,
    pub ts_type: String,
}

/// Main extractor struct
pub struct Extractor {
    options: ExtractOptions,
    result: HashMap<String, TypescriptType>,
}

impl Extractor {
    pub fn new(options: ExtractOptions) -> Self {
        Self {
            options,
            result: HashMap::new(),
        }
    }

    /// Extract TypeScript types from Rust source code
    pub fn extract(&mut self, source: &str) -> Result<HashMap<String, TypescriptType>, Box<dyn std::error::Error>> {
        let syntax_tree = syn::parse_file(source)?;
        
        for item in syntax_tree.items {
            match item {
                syn::Item::Struct(item_struct) => {
                    self.extract_struct(&item_struct)?;
                }
                syn::Item::Enum(item_enum) => {
                    self.extract_enum(&item_enum)?;
                }
                syn::Item::Trait(item_trait) => {
                    self.extract_trait(&item_trait)?;
                }
                _ => {}
            }
        }

        Ok(self.result.clone())
    }

    fn extract_struct(&mut self, item: &ItemStruct) -> Result<(), Box<dyn std::error::Error>> {
        let name = item.ident.to_string();
        let mut fields = Vec::new();

        for field in &item.fields {
            if let Some(field_name) = &field.ident {
                let ts_field = TypescriptField {
                    name: self.convert_field_name(field_name.to_string()),
                    ts_type: self.convert_type(&field.ty)?,
                    optional: self.is_optional_field(field),
                    doc: self.extract_doc(&field.attrs),
                };
                fields.push(ts_field);
            }
        }

        let ts_type = TypescriptType::Interface {
            name: name.clone(),
            fields,
            doc: self.extract_doc(&item.attrs),
        };

        self.result.insert(name, ts_type);
        Ok(())
    }

    fn extract_enum(&mut self, item: &ItemEnum) -> Result<(), Box<dyn std::error::Error>> {
        let name = item.ident.to_string();
        let mut variants = Vec::new();

        for variant in &item.variants {
            variants.push(variant.ident.to_string());
        }

        let ts_type = TypescriptType::Enum {
            name: name.clone(),
            variants,
            doc: self.extract_doc(&item.attrs),
        };

        self.result.insert(name, ts_type);
        Ok(())
    }

    fn extract_trait(&mut self, item: &ItemTrait) -> Result<(), Box<dyn std::error::Error>> {
        let name = item.ident.to_string();
        let mut methods = Vec::new();

        for item in &item.items {
            if let syn::TraitItem::Fn(method) = item {
                let method_name = method.sig.ident.to_string();
                let mut params = Vec::new();

                for (i, input) in method.sig.inputs.iter().enumerate() {
                    match input {
                        syn::FnArg::Typed(pat_type) => {
                            let param_name = format!("arg{}", i);
                            let param_type = self.convert_type(&pat_type.ty)?;
                            params.push(TypescriptParam {
                                name: param_name,
                                ts_type: param_type,
                            });
                        }
                        _ => {}
                    }
                }

                let return_type = match &method.sig.output {
                    syn::ReturnType::Type(_, ty) => self.convert_type(ty)?,
                    _ => "void".to_string(),
                };

                methods.push(TypescriptMethod {
                    name: method_name,
                    params,
                    return_type,
                    doc: self.extract_doc(&method.attrs),
                });
            }
        }

        let ts_type = TypescriptType::Trait {
            name: name.clone(),
            methods,
            doc: self.extract_doc(&item.attrs),
        };

        self.result.insert(name, ts_type);
        Ok(())
    }

    fn convert_type(&self, ty: &Type) -> Result<String, Box<dyn std::error::Error>> {
        match ty {
            Type::Path(type_path) => {
                let path = &type_path.path;
                if let Some(segment) = path.segments.last() {
                    let type_name = segment.ident.to_string();
                    
                    // Handle generic types like Option<T>, Vec<T>
                    if !segment.arguments.is_empty() {
                        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                            if type_name == "Option" && args.args.len() == 1 {
                                if let syn::GenericArgument::Type(inner_ty) = &args.args[0] {
                                    let inner_type = self.convert_type(inner_ty)?;
                                    return Ok(inner_type);
                                }
                            } else if type_name == "Vec" && args.args.len() == 1 {
                                if let syn::GenericArgument::Type(inner_ty) = &args.args[0] {
                                    let inner_type = self.convert_type(inner_ty)?;
                                    return Ok(format!("{}[]", inner_type));
                                }
                            }
                        }
                    }
                    
                    Ok(self.rust_to_typescript_type(&type_name))
                } else {
                    Ok("any".to_string())
                }
            }
            _ => Ok("any".to_string()),
        }
    }

    fn rust_to_typescript_type(&self, rust_type: &str) -> String {
        match rust_type {
            "String" | "str" => "string".to_string(),
            "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "f32" | "f64" | "isize" | "usize" => "number".to_string(),
            "bool" => "boolean".to_string(),
            "Vec" => "Array".to_string(),
            "Option" => "".to_string(), // Handle optionals separately
            _ => rust_type.to_string(),
        }
    }

    fn convert_field_name(&self, name: String) -> String {
        // Convert snake_case to camelCase for JSON compatibility
        let parts: Vec<&str> = name.split('_').collect();
        if parts.len() == 1 {
            return name;
        }
        
        let mut result = parts[0].to_string();
        for part in &parts[1..] {
            if !part.is_empty() {
                let mut chars = part.chars();
                if let Some(first) = chars.next() {
                    result.push(first.to_uppercase().next().unwrap_or(first));
                    result.push_str(&chars.collect::<String>());
                }
            }
        }
        result
    }

    fn is_optional_field(&self, field: &Field) -> bool {
        // Check if field type is Option<T>
        if let Type::Path(type_path) = &field.ty {
            if let Some(segment) = type_path.path.segments.last() {
                if segment.ident == "Option" {
                    return true;
                }
            }
        }
        
        // Check for serde skip_serializing_if attribute
        for attr in &field.attrs {
            if attr.path().is_ident("serde") {
                // This is a simplified check - in practice you'd parse the serde attributes more thoroughly
                let attr_str = format!("{:?}", attr);
                if attr_str.contains("skip_serializing_if") {
                    return true;
                }
            }
        }

        false
    }

    fn extract_doc(&self, attrs: &[syn::Attribute]) -> Option<String> {
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
}

/// Extract TypeScript types from Rust source code
pub fn extract(source: &str, options: ExtractOptions) -> Result<HashMap<String, TypescriptType>, Box<dyn std::error::Error>> {
    let mut extractor = Extractor::new(options);
    extractor.extract(source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_struct() {
        let source = r#"
            pub struct Demo {
                pub foo: String,
                pub bar: u32,
            }
        "#;

        let result = extract(source, ExtractOptions::default()).unwrap();
        assert!(result.contains_key("Demo"));
    }

    #[test]
    fn test_extract_enum() {
        let source = r#"
            pub enum Status {
                Active,
                Inactive,
            }
        "#;

        let result = extract(source, ExtractOptions::default()).unwrap();
        assert!(result.contains_key("Status"));
    }
}
