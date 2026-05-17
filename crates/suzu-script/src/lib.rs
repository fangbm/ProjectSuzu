pub mod compiler;
pub mod extension;
pub mod parser;
pub mod vm;

pub use compiler::{
    compile_document, compile_document_with_extensions, compile_script, migrate_script_source,
    CompileError, CURRENT_SCRIPT_FORMAT_VERSION,
};
pub use extension::{CustomCommandSpec, ExtensionRegistry};
pub use parser::{
    detect_script_syntax, parse_script, parse_script_with_syntax, AstNode, ScriptDocument,
    ScriptSyntax,
};
pub use vm::{
    AnimationKind, ChoiceOption, Command, CommandQueue, CustomCommandAttribute, Position,
    Transition, VisualEffect,
};
