//! This crate reads Liberty format files, commonly used by
//! [EDA](https://en.wikipedia.org/wiki/Electronic_design_automation) tools to describe library
//! cells (including standard cells, hard IP, etc.).
//!
//! # Example
//!
//! ```
//! use liberty_parse::parse_lib;
//!
//! let lib_str = r#"
//! library(sample) {
//!     cell(AND2) {
//!         area: 1;
//!     }
//! }
//! "#;
//!
//! for lib in parse_lib(lib_str).unwrap() {
//!     println!("Library '{}' has {} cells", lib.name, lib.cells.len());
//!     let area = lib
//!         .cells
//!         .get("AND2")
//!         .and_then(|c| c.simple_attribute("area"))
//!         .map_or(-1.0, |v| v.float());
//!     println!("Cell AND2 has area: {}", area);
//! }
//! ```

pub mod ast;
mod error;
pub mod liberty;
mod parser;

pub use ast::{ParseResult, Value};

pub use error::Error;

/// Parse a string slice into a [liberty::Liberty] struct
pub fn parse_lib(contents: &str) -> ParseResult<liberty::Liberty> {
    Ok(liberty::Liberty::from_ast(ast::LibertyAst::from_string(
        contents,
    )?))
}
