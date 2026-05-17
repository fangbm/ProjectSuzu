use crate::parser::{parse_script, AstNode, ScriptDocument};

use super::{
    attributes::{optional, required},
    diagnostics::{span_for, CompileError},
    CURRENT_SCRIPT_FORMAT_VERSION,
};

pub(super) fn migrate_script_source(
    source: &str,
    target_version: u32,
) -> Result<String, CompileError> {
    let document = parse_script(source);
    validate_script_format(&document)?;
    if target_version == CURRENT_SCRIPT_FORMAT_VERSION {
        return Ok(source.to_owned());
    }

    Err(CompileError::UnsupportedScriptVersion {
        version: target_version,
        supported_version: CURRENT_SCRIPT_FORMAT_VERSION,
        span: None,
    })
}

pub(super) fn validate_script_format(document: &ScriptDocument) -> Result<(), CompileError> {
    for (index, node) in document.nodes.iter().enumerate() {
        let AstNode::Command {
            name, attributes, ..
        } = node
        else {
            continue;
        };

        if name != "script" {
            continue;
        }

        let span = span_for(&document.spans, index);
        let version = required(name, attributes, "version")
            .map_err(|error| error.with_span(span))?
            .parse::<u32>()
            .map_err(|_| CompileError::InvalidScriptVersion {
                version: optional(attributes, "version")
                    .unwrap_or_default()
                    .to_owned(),
                span,
            })?;

        if version != CURRENT_SCRIPT_FORMAT_VERSION {
            return Err(CompileError::UnsupportedScriptVersion {
                version,
                supported_version: CURRENT_SCRIPT_FORMAT_VERSION,
                span,
            });
        }
    }

    Ok(())
}
