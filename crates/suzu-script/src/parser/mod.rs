mod ast;
mod braces;
mod document;
mod indent;
mod markup;

pub use ast::{AstNode, Attribute, ScriptDocument, ScriptSyntax, SourceSpan};
pub use document::{detect_script_syntax, parse_script, parse_script_with_syntax};
