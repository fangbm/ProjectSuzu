mod attributes;
mod commands;
mod control_flow;
mod diagnostics;
mod suggestions;
#[cfg(test)]
mod tests;
mod versioning;

use crate::{
    extension::ExtensionRegistry,
    parser::{parse_script, ScriptDocument},
    vm::Command,
};

pub use diagnostics::CompileError;

pub const CURRENT_SCRIPT_FORMAT_VERSION: u32 = 1;

pub fn compile_script(source: &str) -> Result<Vec<Command>, CompileError> {
    compile_document(&parse_script(source))
}

pub fn compile_document(document: &ScriptDocument) -> Result<Vec<Command>, CompileError> {
    compile_document_with_extensions(document, None)
}

pub fn compile_document_with_extensions(
    document: &ScriptDocument,
    extensions: Option<&ExtensionRegistry>,
) -> Result<Vec<Command>, CompileError> {
    versioning::validate_script_format(document)?;
    let (commands, _, _, _) = control_flow::compile_nodes(
        &document.nodes,
        &document.spans,
        0,
        control_flow::StopMode::None,
        None,
        extensions,
    )?;
    Ok(commands)
}

pub fn migrate_script_source(source: &str, target_version: u32) -> Result<String, CompileError> {
    versioning::migrate_script_source(source, target_version)
}
