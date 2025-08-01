//! TypeScript type definitions and utilities

use serde::{Deserialize, Serialize};

/// TypeScript primitive types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TsPrimitive {
    String,
    Number,
    Boolean,
    Any,
    Void,
    Null,
    Undefined,
}

impl TsPrimitive {
    pub fn as_str(&self) -> &'static str {
        match self {
            TsPrimitive::String => "string",
            TsPrimitive::Number => "number",
            TsPrimitive::Boolean => "boolean",
            TsPrimitive::Any => "any",
            TsPrimitive::Void => "void",
            TsPrimitive::Null => "null",
            TsPrimitive::Undefined => "undefined",
        }
    }
}

/// TypeScript type representation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TsType {
    Primitive(TsPrimitive),
    Array(Box<TsType>),
    Object(Vec<TsProperty>),
    Union(Vec<TsType>),
    Reference(String),
    Function {
        params: Vec<TsParameter>,
        return_type: Box<TsType>,
    },
    Generic {
        base: String,
        args: Vec<TsType>,
    },
}

/// TypeScript object property
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TsProperty {
    pub name: String,
    pub ts_type: TsType,
    pub optional: bool,
    pub readonly: bool,
}

/// TypeScript function parameter
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TsParameter {
    pub name: String,
    pub ts_type: TsType,
    pub optional: bool,
}

impl TsType {
    /// Convert to TypeScript string representation
    pub fn to_typescript(&self) -> String {
        match self {
            TsType::Primitive(p) => p.as_str().to_string(),
            TsType::Array(inner) => format!("{}[]", inner.to_typescript()),
            TsType::Object(props) => {
                let prop_strings: Vec<String> = props.iter().map(|p| {
                    let optional = if p.optional { "?" } else { "" };
                    let readonly = if p.readonly { "readonly " } else { "" };
                    format!("{}{}{}: {}", readonly, p.name, optional, p.ts_type.to_typescript())
                }).collect();
                format!("{{ {} }}", prop_strings.join(", "))
            }
            TsType::Union(types) => {
                let type_strings: Vec<String> = types.iter().map(|t| t.to_typescript()).collect();
                type_strings.join(" | ")
            }
            TsType::Reference(name) => name.clone(),
            TsType::Function { params, return_type } => {
                let param_strings: Vec<String> = params.iter().map(|p| {
                    let optional = if p.optional { "?" } else { "" };
                    format!("{}{}: {}", p.name, optional, p.ts_type.to_typescript())
                }).collect();
                format!("({}) => {}", param_strings.join(", "), return_type.to_typescript())
            }
            TsType::Generic { base, args } => {
                let arg_strings: Vec<String> = args.iter().map(|a| a.to_typescript()).collect();
                format!("{}<{}>", base, arg_strings.join(", "))
            }
        }
    }

    /// Create a string type
    pub fn string() -> Self {
        TsType::Primitive(TsPrimitive::String)
    }

    /// Create a number type
    pub fn number() -> Self {
        TsType::Primitive(TsPrimitive::Number)
    }

    /// Create a boolean type
    pub fn boolean() -> Self {
        TsType::Primitive(TsPrimitive::Boolean)
    }

    /// Create an array type
    pub fn array(inner: TsType) -> Self {
        TsType::Array(Box::new(inner))
    }

    /// Create a reference type
    pub fn reference(name: impl Into<String>) -> Self {
        TsType::Reference(name.into())
    }

    /// Create an optional type (union with undefined)
    pub fn optional(self) -> Self {
        TsType::Union(vec![self, TsType::Primitive(TsPrimitive::Undefined)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_to_typescript() {
        assert_eq!(TsType::string().to_typescript(), "string");
        assert_eq!(TsType::number().to_typescript(), "number");
        assert_eq!(TsType::boolean().to_typescript(), "boolean");
    }

    #[test]
    fn test_array_to_typescript() {
        let array_type = TsType::array(TsType::string());
        assert_eq!(array_type.to_typescript(), "string[]");
    }

    #[test]
    fn test_object_to_typescript() {
        let obj_type = TsType::Object(vec![
            TsProperty {
                name: "foo".to_string(),
                ts_type: TsType::string(),
                optional: true,
                readonly: false,
            },
            TsProperty {
                name: "bar".to_string(),
                ts_type: TsType::number(),
                optional: false,
                readonly: true,
            },
        ]);
        let result = obj_type.to_typescript();
        assert!(result.contains("foo?: string"));
        assert!(result.contains("readonly bar: number"));
    }

    #[test]
    fn test_union_to_typescript() {
        let union_type = TsType::Union(vec![
            TsType::string(),
            TsType::number(),
        ]);
        assert_eq!(union_type.to_typescript(), "string | number");
    }
}
