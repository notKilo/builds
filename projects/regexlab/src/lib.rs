//! regexlab: a from-scratch regular expression engine.

pub mod ast;
pub mod parser;

pub use parser::{parse, ParseError};
