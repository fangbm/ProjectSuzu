use crate::{
    extension::ExtensionRegistry,
    parser::{AstNode, SourceSpan},
    vm::{ChoiceOption, Command},
};

use super::{
    attributes::{optional, required},
    commands::compile_command,
    diagnostics::{span_for, CompileError},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StopMode {
    None,
    IfBody,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StopToken {
    End,
    Else,
    EndIf,
}

pub(super) fn compile_nodes(
    nodes: &[AstNode],
    spans: &[SourceSpan],
    start: usize,
    stop_mode: StopMode,
    mut current_speaker: Option<String>,
    extensions: Option<&ExtensionRegistry>,
) -> Result<(Vec<Command>, usize, Option<String>, StopToken), CompileError> {
    let mut commands = Vec::new();
    let mut index = start;

    while let Some(node) = nodes.get(index) {
        match node {
            AstNode::Speaker(name) => current_speaker = Some(name.clone()),
            AstNode::Text(content) => commands.push(Command::Text {
                speaker: current_speaker.clone(),
                content: content.clone(),
            }),
            AstNode::Label(name) => commands.push(Command::Label { name: name.clone() }),
            AstNode::Command { name, .. } if name == "script" => {}
            AstNode::Command { name, .. } if name == "choice" => {
                let (choice, next_index) = compile_choice_group(nodes, spans, index)?;
                commands.push(choice);
                index = next_index;
                continue;
            }
            AstNode::Command {
                name, attributes, ..
            } if name == "if" => {
                let condition = compile_if_condition(name, attributes)
                    .map_err(|error| error.with_span(span_for(spans, index)))?;
                let (then_commands, next_index, _, stop_token) = compile_nodes(
                    nodes,
                    spans,
                    index + 1,
                    StopMode::IfBody,
                    current_speaker.clone(),
                    extensions,
                )?;
                let (else_commands, next_index) = if stop_token == StopToken::Else {
                    let (else_commands, else_end_index, _, _) = compile_nodes(
                        nodes,
                        spans,
                        next_index + 1,
                        StopMode::IfBody,
                        current_speaker.clone(),
                        extensions,
                    )?;
                    (else_commands, else_end_index)
                } else {
                    (Vec::new(), next_index)
                };
                commands.push(Command::If {
                    condition,
                    then_commands,
                    else_commands,
                });
                index = next_index + 1;
                continue;
            }
            AstNode::Command { name, .. } if name == "else" && stop_mode == StopMode::IfBody => {
                return Ok((commands, index, current_speaker, StopToken::Else));
            }
            AstNode::Command { name, .. } if name == "endif" && stop_mode == StopMode::IfBody => {
                return Ok((commands, index, current_speaker, StopToken::EndIf));
            }
            AstNode::Command {
                name,
                args,
                attributes,
            } => commands.push(
                compile_command(name, args, attributes, extensions)
                    .map_err(|error| error.with_span(span_for(spans, index)))?,
            ),
            AstNode::Comment(_) => {}
        }
        index += 1;
    }

    Ok((commands, index, current_speaker, StopToken::End))
}

fn compile_choice_group(
    nodes: &[AstNode],
    spans: &[SourceSpan],
    start: usize,
) -> Result<(Command, usize), CompileError> {
    let mut options = Vec::new();
    let mut index = start;

    while let Some(AstNode::Command {
        name,
        args,
        attributes,
    }) = nodes.get(index)
    {
        if name != "choice" {
            break;
        }

        options.push(ChoiceOption {
            text: args.first().cloned().unwrap_or_default(),
            goto: required(name, attributes, "goto")
                .map_err(|error| error.with_span(span_for(spans, index)))?,
            condition: optional(attributes, "cond").map(ToOwned::to_owned),
        });
        index += 1;
    }

    Ok((Command::Choice { options }, index))
}

fn compile_if_condition(
    command: &str,
    attributes: &[crate::parser::Attribute],
) -> Result<String, CompileError> {
    if let Some(condition) = optional(attributes, "cond") {
        return Ok(condition.to_owned());
    }

    let var = required(command, attributes, "var")?;
    let op = optional(attributes, "op").unwrap_or("eq");
    let value = required(command, attributes, "value")?;
    Ok(format!("{var}{}{}", compare_operator(op), value))
}

fn compare_operator(op: &str) -> &str {
    match op {
        "gt" => ">",
        "ge" | "gte" => ">=",
        "lt" => "<",
        "le" | "lte" => "<=",
        "ne" | "neq" => "!=",
        "eq" => "==",
        other => other,
    }
}
