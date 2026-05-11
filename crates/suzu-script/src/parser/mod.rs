mod ast;
mod document;

pub use ast::{AstNode, Attribute, ScriptDocument, SourceSpan};
pub use document::parse_script;
