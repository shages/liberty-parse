//! Defines the types and parser for the Abstract Syntax Tree (AST) representation of a Liberty
//! file.
//!

use std::result;

use crate::error::Error;
use crate::liberty::Liberty;
use crate::parser::parse_libs;

use itertools::Itertools;
use nom::error::VerboseError;

/// Result type for parsing
pub type ParseResult<'a, T> = result::Result<T, Error<'a>>;

/// Liberty file AST representation
///
/// Each liberty file can have one or more `library`s defined in it, which are represented as a
/// [`GroupItem::Group`] variant.
#[derive(Debug)]
pub struct LibertyAst(pub Vec<GroupItem>);

impl LibertyAst {
    /// Create a new AST from a vector of `GroupItem`s
    pub fn new(libs: Vec<GroupItem>) -> Self {
        Self(libs)
    }

    /// Parse a Liberty file's string representation into the AST
    pub fn from_string(input: &str) -> ParseResult<Self> {
        parse_libs::<VerboseError<&str>>(input)
            .map_err(|e| Error::new(input, e))
            .map(|(_, libs)| LibertyAst::new(libs))
    }

    /// Get a `String` representation from an AST
    pub fn to_string(&self) -> String {
        items_to_string(&self.0)
    }

    /// Convert an AST into a [`Liberty`] struct
    pub fn into_liberty(self) -> Liberty {
        Liberty::from_ast(self)
    }

    /// Convert a [`Liberty`] struct into an AST
    pub fn from_liberty(lib: Liberty) -> Self {
        lib.to_ast()
    }
}

// Recursively convert a vector of [`GroupItem`]s into a single `String`
fn items_to_string(items: &Vec<GroupItem>) -> String {
    items
        .iter()
        .map(|item| match item {
            GroupItem::SimpleAttr(name, value) => format!("{} : {};\n", name, value.to_string()),
            GroupItem::ComplexAttr(name, values) => format!(
                "{} (\n{}\n)\n",
                name,
                values.iter().map(|v| v.to_string()).join(", ")
            ),
            GroupItem::Comment(v) => format!("/*\n{}\n*/", v),
            GroupItem::Group(type_, name, group_items) => format!(
                "{} ( {} ) {{\n{}\n}}",
                type_,
                name,
                items_to_string(group_items)
            ),
        })
        .join("\n")
}

/// Intermediate representation
#[derive(Debug, PartialEq, Clone)]
pub enum GroupItem {
    // type, name, values
    Group(String, String, Vec<GroupItem>),
    // name, value
    SimpleAttr(String, Value),
    ComplexAttr(String, Vec<Value>),
    // contents
    Comment(String),
}

impl GroupItem {
    /// Convert [`Value::Float`] to `f64` or panic
    pub fn group(&self) -> (String, String, Vec<GroupItem>) {
        if let GroupItem::Group(type_, name, items) = self {
            (String::from(type_), String::from(name), items.clone())
        } else {
            panic!("Not variant GroupItem::Group");
        }
    }
}

/// Liberty value type
///
/// A wide range of types are defined for the Liberty syntax. Because there is little to no way
/// to parse enumerated types from the syntax alone, enumerated types are parsed as the
/// [`Value::Expression`] variant.
#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    /// Boolean value, parsed from the keywords `true` and `false`
    Bool(bool),
    /// Floating point value.
    ///
    /// All numbers are parsed into `f64`. While the Liberty specification differentiates between
    /// integers and floating point values on a per-field basis, all are parsed into an `f64`.
    Float(f64),
    /// Group of floating point values in quotation marks
    ///
    /// For example, this complex attribute
    ///
    /// ```text
    /// values ( \
    ///   "1.0, 2.0, 3.0", \
    ///   "4.0, 5.0, 6.0" \
    /// );
    /// ```
    ///
    /// will be parsed into a `Vec<Value::FloatGroup>`.
    FloatGroup(Vec<f64>),
    /// String enclosed in quotation marks
    String(String),
    /// Expression
    ///
    /// Enumerated values, such as the `delay_model` simple attribute,  are parsed as a
    /// [`Value::Expression`].
    Expression(String),
}

impl Value {
    /// Get a `String` representation of a [`Value`]
    pub fn to_string(&self) -> String {
        match self {
            Value::String(v) => format!("\"{}\"", v),
            Value::Expression(v) => format!("{}", v),
            Value::Bool(v) => {
                if *v {
                    "true".to_string()
                } else {
                    "false".to_string()
                }
            }
            Value::Float(v) => format!("{:.6}", v),
            Value::FloatGroup(v) => format!("\"{}\"", format!("{:.6}", v.iter().format(", "))),
        }
    }

    /// Convert [`Value::Float`] to `f64` or panic
    pub fn float(&self) -> f64 {
        if let Value::Float(v) = self {
            v.clone()
        } else {
            panic!("Not a float")
        }
    }

    /// Convert [`Value::String`] to `String` or panic
    pub fn string(&self) -> String {
        if let Value::String(v) = self {
            v.clone()
        } else {
            panic!("Not a string")
        }
    }

    /// Convert [`Value::Expression`] to `String` or panic
    pub fn expr(&self) -> String {
        if let Value::Expression(v) = self {
            v.clone()
        } else {
            panic!("Not a string")
        }
    }

    /// Convert [`Value::Bool`] to `bool` or panic
    pub fn bool(&self) -> bool {
        if let Value::Bool(v) = self {
            v.clone()
        } else {
            panic!("Not a bool")
        }
    }

    /// Convert [`Value::FloatGroup`] to `Vec<f64>` or panic
    pub fn float_group(&self) -> Vec<f64> {
        if let Value::FloatGroup(v) = self {
            v.clone()
        } else {
            panic!("Not a float group")
        }
    }
}

#[cfg(test)]
mod test {
    use super::{LibertyAst, Value};

    macro_rules! parse_file {
        ($fname:ident) => {{
            let data = include_str!(concat!("../data/", stringify!($fname), ".lib"));
            LibertyAst::from_string(data).unwrap()
        }};
    }

    #[test]
    fn test_files() {
        parse_file!(small);
        parse_file!(cells);
        parse_file!(cells_timing);
    }

    #[test]
    fn test_values() {
        assert_eq!(Value::Bool(false).bool(), false);
        assert_eq!(Value::Float(-3.45).float(), -3.45f64);
        assert_eq!(Value::Expression("A & B".to_string()).expr(), "A & B");
        assert_eq!(
            Value::FloatGroup(vec![1.2, 3.4]).float_group(),
            vec![1.2, 3.4]
        );
        assert_eq!(Value::String("abc def".to_string()).string(), "abc def");
    }
}
