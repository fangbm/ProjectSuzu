use std::{error::Error, fmt};

use suzu_core::{Color, Vec2};

use crate::{
    extension::ExtensionRegistry,
    parser::{parse_script, AstNode, Attribute, ScriptDocument, SourceSpan},
    vm::{
        Animation, AnimationKind, Command, CustomCommandAttribute, Position, Transition,
        VisualEffect,
    },
};

pub const CURRENT_SCRIPT_FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileError {
    MissingAttribute {
        command: String,
        key: String,
        span: Option<SourceSpan>,
    },
    UnknownCommand {
        command: String,
        span: Option<SourceSpan>,
        suggestion: Option<String>,
    },
    InvalidScriptVersion {
        version: String,
        span: Option<SourceSpan>,
    },
    UnsupportedScriptVersion {
        version: u32,
        supported_version: u32,
        span: Option<SourceSpan>,
    },
}

impl CompileError {
    fn with_span(self, span: Option<SourceSpan>) -> Self {
        match self {
            Self::MissingAttribute {
                command,
                key,
                span: existing,
            } => Self::MissingAttribute {
                command,
                key,
                span: existing.or(span),
            },
            Self::UnknownCommand {
                command,
                span: existing,
                suggestion,
            } => Self::UnknownCommand {
                command,
                span: existing.or(span),
                suggestion,
            },
            Self::InvalidScriptVersion {
                version,
                span: existing,
            } => Self::InvalidScriptVersion {
                version,
                span: existing.or(span),
            },
            Self::UnsupportedScriptVersion {
                version,
                supported_version,
                span: existing,
            } => Self::UnsupportedScriptVersion {
                version,
                supported_version,
                span: existing.or(span),
            },
        }
    }

    pub fn span(&self) -> Option<SourceSpan> {
        match self {
            Self::MissingAttribute { span, .. }
            | Self::UnknownCommand { span, .. }
            | Self::InvalidScriptVersion { span, .. }
            | Self::UnsupportedScriptVersion { span, .. } => *span,
        }
    }
}

impl fmt::Display for CompileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(span) = self.span() {
            write!(formatter, "line {}, column {}: ", span.line, span.column)?;
        }

        match self {
            Self::MissingAttribute { command, key, .. } => {
                write!(
                    formatter,
                    "missing required attribute `{key}` for @{command}"
                )
            }
            Self::UnknownCommand {
                command,
                suggestion,
                ..
            } => {
                write!(formatter, "unknown command @{command}")?;
                if let Some(suggestion) = suggestion {
                    write!(formatter, "; did you mean @{suggestion}?")?;
                }
                Ok(())
            }
            Self::InvalidScriptVersion { version, .. } => {
                write!(formatter, "invalid script format version `{version}`")
            }
            Self::UnsupportedScriptVersion {
                version,
                supported_version,
                ..
            } => {
                write!(
                    formatter,
                    "unsupported script format version {version}; supported version is {supported_version}"
                )
            }
        }
    }
}

impl Error for CompileError {}

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
    validate_script_format(document)?;
    let (commands, _, _, _) = compile_nodes(
        &document.nodes,
        &document.spans,
        0,
        StopMode::None,
        None,
        extensions,
    )?;
    Ok(commands)
}

pub fn migrate_script_source(source: &str, target_version: u32) -> Result<String, CompileError> {
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

fn validate_script_format(document: &ScriptDocument) -> Result<(), CompileError> {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StopMode {
    None,
    IfBody,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StopToken {
    End,
    Else,
    EndIf,
}

fn compile_nodes(
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
            AstNode::Command {
                name,
                args,
                attributes,
            } if name == "choice" => {
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

        options.push(crate::vm::ChoiceOption {
            text: args.first().cloned().unwrap_or_default(),
            goto: required(name, attributes, "goto")
                .map_err(|error| error.with_span(span_for(spans, index)))?,
            condition: optional(attributes, "cond").map(ToOwned::to_owned),
        });
        index += 1;
    }

    Ok((Command::Choice { options }, index))
}

fn compile_command(
    name: &str,
    args: &[String],
    attributes: &[Attribute],
    extensions: Option<&ExtensionRegistry>,
) -> Result<Command, CompileError> {
    match name {
        "bg" => {
            let file = required(name, attributes, "file")?;
            let time_ms = optional_u32(attributes, "time").unwrap_or(0);
            let method = match optional(attributes, "method") {
                Some("crossfade") => Transition::CrossFade {
                    duration_ms: time_ms,
                },
                Some("fade")
                | Some("fadecolor")
                | Some("fade_color")
                | Some("fade-through-color")
                | Some("fade_through_color") => Transition::FadeThroughColor {
                    color: optional_color(attributes, "color").unwrap_or(Color::BLACK),
                    duration_ms: time_ms,
                },
                _ => Transition::Instant,
            };
            Ok(Command::Bg {
                file,
                time_ms,
                method,
            })
        }
        "char" => {
            let name_attr = required(name, attributes, "name")?;
            let face = optional(attributes, "face").unwrap_or("neutral").to_owned();
            let pos = compile_position(attributes);
            let size = compile_size(attributes, Vec2::new(360.0, 720.0));
            let flip_x = optional_bool(attributes, "flip")
                .or_else(|| optional_bool(attributes, "flip_x"))
                .unwrap_or(false);
            let layer = optional_i32(attributes, "layer").unwrap_or(0);
            Ok(Command::Char {
                name: name_attr,
                face,
                pos,
                size,
                flip_x,
                layer,
            })
        }
        "hidechar" | "hide" => Ok(Command::HideChar {
            name: required(name, attributes, "name")?,
        }),
        "jump" => Ok(Command::Jump {
            label: required(name, attributes, "goto")?,
        }),
        "call" => Ok(Command::Call {
            label: required(name, attributes, "goto")?,
        }),
        "return" => Ok(Command::Return),
        "set" | "var" => Ok(Command::SetVar {
            name: required(name, attributes, "name")?,
            value: required(name, attributes, "value")?,
        }),
        "savename" => Ok(Command::SaveName {
            text: required(name, attributes, "text")?,
        }),
        "autosave" => Ok(Command::AutoSave),
        "choice" => Ok(Command::Choice {
            options: vec![crate::vm::ChoiceOption {
                text: args.first().cloned().unwrap_or_default(),
                goto: required(name, attributes, "goto")?,
                condition: optional(attributes, "cond").map(ToOwned::to_owned),
            }],
        }),
        "playbgm" => Ok(Command::PlayBgm {
            file: required(name, attributes, "file")?,
            looping: optional_bool(attributes, "loop").unwrap_or(true),
            fadein_ms: optional_u32(attributes, "fadein").unwrap_or(0),
        }),
        "stopbgm" => Ok(Command::StopBgm {
            fadeout_ms: optional_u32(attributes, "fadeout").unwrap_or(0),
        }),
        "playvoice" => Ok(Command::PlayVoice {
            file: required(name, attributes, "file")?,
            fadein_ms: optional_u32(attributes, "fadein").unwrap_or(0),
        }),
        "voice" => Ok(Command::CueVoice {
            file: required(name, attributes, "file")?,
            fadein_ms: optional_u32(attributes, "fadein").unwrap_or(0),
        }),
        "stopvoice" => Ok(Command::StopVoice {
            fadeout_ms: optional_u32(attributes, "fadeout").unwrap_or(0),
        }),
        "wait" => Ok(Command::Wait {
            duration_ms: optional_u32(attributes, "time").unwrap_or(0),
        }),
        "hidemsg" | "hidemessage" => Ok(Command::MessageBox { visible: false }),
        "showmsg" | "showmessage" => Ok(Command::MessageBox { visible: true }),
        "anim" => Ok(Command::Anim {
            target: required(name, attributes, "target")?,
            animation: compile_animation(name, attributes),
        }),
        "fx" => Ok(Command::Fx {
            effect: compile_fx(name, attributes),
        }),
        other if extensions.is_some_and(|registry| registry.contains_command(other)) => {
            Ok(Command::Custom {
                name: other.to_owned(),
                args: args.to_vec(),
                attributes: attributes
                    .iter()
                    .map(|attribute| CustomCommandAttribute {
                        key: attribute.key.clone(),
                        value: attribute.value.clone(),
                    })
                    .collect(),
            })
        }
        other => Err(CompileError::UnknownCommand {
            command: other.to_owned(),
            span: None,
            suggestion: suggest_command(other).map(ToOwned::to_owned),
        }),
    }
}

fn compile_position(attributes: &[Attribute]) -> Position {
    match (optional_f32(attributes, "x"), optional_f32(attributes, "y")) {
        (Some(x), Some(y)) => Position::Custom(Vec2::new(x, y)),
        _ => match optional(attributes, "pos").unwrap_or("center") {
            "left" => Position::Left,
            "right" => Position::Right,
            _ => Position::Center,
        },
    }
}

fn compile_size(attributes: &[Attribute], default: Vec2) -> Vec2 {
    Vec2::new(
        optional_f32(attributes, "width")
            .or_else(|| optional_f32(attributes, "w"))
            .unwrap_or(default.x),
        optional_f32(attributes, "height")
            .or_else(|| optional_f32(attributes, "h"))
            .unwrap_or(default.y),
    )
}

fn compile_if_condition(command: &str, attributes: &[Attribute]) -> Result<String, CompileError> {
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

fn compile_animation(command: &str, attributes: &[Attribute]) -> Animation {
    let duration_ms = optional_u32(attributes, "duration").unwrap_or(0);
    let kind = match optional(attributes, "type").unwrap_or("shake") {
        "move" | "move_to" => AnimationKind::MoveTo {
            position: Vec2::new(
                optional_f32(attributes, "x").unwrap_or(0.0),
                optional_f32(attributes, "y").unwrap_or(0.0),
            ),
        },
        "zoom" => AnimationKind::Zoom {
            center: Vec2::new(
                optional_f32(attributes, "center_x").unwrap_or(0.5),
                optional_f32(attributes, "center_y").unwrap_or(0.5),
            ),
            scale: optional_f32(attributes, "scale").unwrap_or(1.0),
        },
        "fade" | "fade_to" => AnimationKind::FadeTo {
            opacity: optional_f32(attributes, "opacity").unwrap_or(1.0),
        },
        _ => AnimationKind::Shake {
            intensity: optional_f32(attributes, "intensity").unwrap_or(1.0),
        },
    };

    let _ = command;
    Animation { kind, duration_ms }
}

fn compile_fx(command: &str, attributes: &[Attribute]) -> VisualEffect {
    let duration_ms = optional_u32(attributes, "duration").unwrap_or(0);
    let effect = match optional(attributes, "type").unwrap_or("flash") {
        "quake" => VisualEffect::Quake {
            intensity: optional_f32(attributes, "intensity").unwrap_or(1.0),
            duration_ms,
        },
        _ => VisualEffect::Flash {
            color: optional_color(attributes, "color").unwrap_or(Color::WHITE),
            duration_ms,
        },
    };

    let _ = command;
    effect
}

fn required(command: &str, attributes: &[Attribute], key: &str) -> Result<String, CompileError> {
    optional(attributes, key)
        .map(ToOwned::to_owned)
        .ok_or_else(|| CompileError::MissingAttribute {
            command: command.to_owned(),
            key: key.to_owned(),
            span: None,
        })
}

fn span_for(spans: &[SourceSpan], index: usize) -> Option<SourceSpan> {
    spans.get(index).copied()
}

fn suggest_command(command: &str) -> Option<&'static str> {
    known_commands()
        .iter()
        .copied()
        .map(|known| (known, edit_distance(command, known)))
        .filter(|(_known, distance)| *distance <= 3)
        .min_by_key(|(_known, distance)| *distance)
        .map(|(known, _distance)| known)
}

fn known_commands() -> &'static [&'static str] {
    &[
        "anim",
        "autosave",
        "bg",
        "call",
        "char",
        "choice",
        "else",
        "endif",
        "fx",
        "hidechar",
        "hidemsg",
        "if",
        "jump",
        "playbgm",
        "playvoice",
        "return",
        "savename",
        "set",
        "showmsg",
        "stopbgm",
        "stopvoice",
        "voice",
        "wait",
    ]
}

fn edit_distance(left: &str, right: &str) -> usize {
    let mut previous = (0..=right.chars().count()).collect::<Vec<_>>();
    let mut current = vec![0; previous.len()];

    for (left_index, left_ch) in left.chars().enumerate() {
        current[0] = left_index + 1;
        for (right_index, right_ch) in right.chars().enumerate() {
            let deletion = previous[right_index + 1] + 1;
            let insertion = current[right_index] + 1;
            let substitution = previous[right_index] + usize::from(left_ch != right_ch);
            current[right_index + 1] = deletion.min(insertion).min(substitution);
        }
        std::mem::swap(&mut previous, &mut current);
    }

    previous[right.chars().count()]
}

fn optional<'a>(attributes: &'a [Attribute], key: &str) -> Option<&'a str> {
    attributes
        .iter()
        .find(|attribute| attribute.key == key)
        .map(|attribute| attribute.value.as_str())
}

fn optional_u32(attributes: &[Attribute], key: &str) -> Option<u32> {
    optional(attributes, key)?.parse().ok()
}

fn optional_i32(attributes: &[Attribute], key: &str) -> Option<i32> {
    optional(attributes, key)?.parse().ok()
}

fn optional_f32(attributes: &[Attribute], key: &str) -> Option<f32> {
    optional(attributes, key)?.parse().ok()
}

fn optional_bool(attributes: &[Attribute], key: &str) -> Option<bool> {
    optional(attributes, key)?.parse().ok()
}

fn optional_color(attributes: &[Attribute], key: &str) -> Option<Color> {
    let value = optional(attributes, key)?.trim_start_matches('#');
    if value.len() != 6 {
        return None;
    }

    let r = u8::from_str_radix(&value[0..2], 16).ok()? as f32 / 255.0;
    let g = u8::from_str_radix(&value[2..4], 16).ok()? as f32 / 255.0;
    let b = u8::from_str_radix(&value[4..6], 16).ok()? as f32 / 255.0;
    Some(Color::rgba(r, g, b, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiles_dialogue_with_speaker() {
        let commands = compile_script("# 艾琳\n你好").unwrap();
        assert!(matches!(
            &commands[0],
            Command::Text {
                speaker: Some(name),
                content
            } if name == "艾琳" && content == "你好"
        ));
    }

    #[test]
    fn script_version_header_is_metadata() {
        let commands = compile_script("@script version=1\n# N\nHello").unwrap();

        assert_eq!(commands.len(), 1);
        assert!(matches!(&commands[0], Command::Text { content, .. } if content == "Hello"));
    }

    #[test]
    fn unsupported_script_version_reports_span() {
        let error = compile_script("@script version=2\n# N\nHello").unwrap_err();

        assert!(matches!(
            error,
            CompileError::UnsupportedScriptVersion {
                version: 2,
                supported_version: CURRENT_SCRIPT_FORMAT_VERSION,
                span: Some(SourceSpan { line: 1, column: 1 }),
            }
        ));
    }

    #[test]
    fn invalid_script_version_reports_span() {
        let error = compile_script("@script version=next").unwrap_err();

        assert!(matches!(
            error,
            CompileError::InvalidScriptVersion {
                ref version,
                span: Some(SourceSpan { line: 1, column: 1 }),
            } if version == "next"
        ));
    }

    #[test]
    fn script_migration_noops_current_version() {
        let source = "@script version=1\n# N\nHello";

        assert_eq!(
            migrate_script_source(source, CURRENT_SCRIPT_FORMAT_VERSION).unwrap(),
            source
        );
    }

    #[test]
    fn compiles_background_command() {
        let commands = compile_script("@bg file=\"school\" time=800 method=crossfade").unwrap();
        assert!(matches!(
            &commands[0],
            Command::Bg {
                file,
                method: Transition::CrossFade { duration_ms: 800 },
                ..
            } if file == "school"
        ));
    }

    #[test]
    fn unknown_command_reports_span_and_suggestion() {
        let error = compile_script("# N\n@bgg file=\"school\"").unwrap_err();

        assert!(matches!(
            error,
            CompileError::UnknownCommand {
                ref command,
                span: Some(SourceSpan { line: 2, column: 1 }),
                suggestion: Some(ref suggestion),
            } if command == "bgg" && suggestion == "bg"
        ));
        assert_eq!(
            error.to_string(),
            "line 2, column 1: unknown command @bgg; did you mean @bg?"
        );
    }

    #[test]
    fn missing_attribute_reports_span() {
        let error = compile_script("  @char face=\"happy\"").unwrap_err();

        assert!(matches!(
            error,
            CompileError::MissingAttribute {
                ref command,
                ref key,
                span: Some(SourceSpan { line: 1, column: 3 }),
            } if command == "char" && key == "name"
        ));
    }

    #[test]
    fn compiles_background_fade_through_color_command() {
        let commands =
            compile_script("@bg file=\"school\" time=800 method=fade_through_color color=#112233")
                .unwrap();

        assert!(matches!(
            &commands[0],
            Command::Bg {
                file,
                method: Transition::FadeThroughColor { color, duration_ms: 800 },
                ..
            } if file == "school" && *color == Color::rgba(0x11 as f32 / 255.0, 0x22 as f32 / 255.0, 0x33 as f32 / 255.0, 1.0)
        ));
    }

    #[test]
    fn compiles_character_custom_position() {
        let commands = compile_script("@char name=\"eileen\" x=320 y=24 layer=4").unwrap();

        assert!(matches!(
            &commands[0],
            Command::Char {
                name,
                pos: Position::Custom(position),
                layer: 4,
                ..
            } if name == "eileen" && *position == Vec2::new(320.0, 24.0)
        ));
    }

    #[test]
    fn compiles_character_custom_size() {
        let commands =
            compile_script("@char name=\"eileen\" width=420 height=680 layer=4").unwrap();

        assert!(matches!(
            &commands[0],
            Command::Char {
                name,
                size,
                layer: 4,
                ..
            } if name == "eileen" && *size == Vec2::new(420.0, 680.0)
        ));
    }

    #[test]
    fn compiles_character_flip() {
        let commands = compile_script("@char name=\"eileen\" flip=true").unwrap();

        assert!(matches!(
            &commands[0],
            Command::Char {
                name,
                flip_x: true,
                ..
            } if name == "eileen"
        ));
    }

    #[test]
    fn compiles_animation_command() {
        let commands =
            compile_script("@anim target=\"eileen\" type=zoom scale=1.2 duration=1000").unwrap();

        assert!(matches!(
            &commands[0],
            Command::Anim {
                target,
                animation: Animation {
                    kind: AnimationKind::Zoom { scale, .. },
                    duration_ms: 1000
                }
            } if target == "eileen" && (*scale - 1.2).abs() < 0.0001
        ));
    }

    #[test]
    fn compiles_fade_animation_command() {
        let commands =
            compile_script("@anim target=\"eileen\" type=fade opacity=0.25 duration=500").unwrap();

        assert!(matches!(
            &commands[0],
            Command::Anim {
                target,
                animation: Animation {
                    kind: AnimationKind::FadeTo { opacity },
                    duration_ms: 500
                }
            } if target == "eileen" && (*opacity - 0.25).abs() < 0.0001
        ));
    }

    #[test]
    fn compiles_choice_group_and_label() {
        let commands =
            compile_script("@choice \"去教室\" goto=classroom\n@choice \"去天台\" goto=roof\n*classroom\n# 艾琳\n走吧")
                .unwrap();

        assert!(matches!(
            &commands[0],
            Command::Choice { options } if options.len() == 2 && options[1].goto == "roof"
        ));
        assert!(matches!(&commands[1], Command::Label { name } if name == "classroom"));
    }

    #[test]
    fn compiles_choice_condition() {
        let commands =
            compile_script("@choice \"艾琳路线\" goto=eileen cond=affection_eileen>=50").unwrap();

        assert!(matches!(
            &commands[0],
            Command::Choice { options } if options[0].condition.as_deref() == Some("affection_eileen>=50")
        ));
    }

    #[test]
    fn compiles_set_variable_command() {
        let commands = compile_script("@set name=affection_eileen value=52").unwrap();

        assert!(matches!(
            &commands[0],
            Command::SetVar { name, value } if name == "affection_eileen" && value == "52"
        ));
    }

    #[test]
    fn compiles_call_and_return_commands() {
        let commands = compile_script("@call goto=common\n@return").unwrap();

        assert!(matches!(
            &commands[0],
            Command::Call { label } if label == "common"
        ));
        assert!(matches!(&commands[1], Command::Return));
    }

    #[test]
    fn compiles_hide_character_command() {
        let commands = compile_script("@hidechar name=\"eileen\"").unwrap();

        assert!(matches!(
            &commands[0],
            Command::HideChar { name } if name == "eileen"
        ));
    }

    #[test]
    fn compiles_wait_command() {
        let commands = compile_script("@wait time=750").unwrap();

        assert!(matches!(&commands[0], Command::Wait { duration_ms: 750 }));
    }

    #[test]
    fn compiles_voice_commands() {
        let commands = compile_script(
            "@playvoice file=\"voices/eileen_001.ogg\" fadein=120\n@voice file=\"voices/eileen_002.ogg\"\n@stopvoice fadeout=80",
        )
        .unwrap();

        assert!(matches!(
            &commands[0],
            Command::PlayVoice { file, fadein_ms: 120 } if file == "voices/eileen_001.ogg"
        ));
        assert!(matches!(
            &commands[1],
            Command::CueVoice { file, fadein_ms: 0 } if file == "voices/eileen_002.ogg"
        ));
        assert!(matches!(
            &commands[2],
            Command::StopVoice { fadeout_ms: 80 }
        ));
    }

    #[test]
    fn compiles_message_box_visibility_commands() {
        let commands = compile_script("@hidemsg\n@showmsg").unwrap();

        assert!(matches!(
            &commands[0],
            Command::MessageBox { visible: false }
        ));
        assert!(matches!(
            &commands[1],
            Command::MessageBox { visible: true }
        ));
    }

    #[test]
    fn compiles_if_block_with_nested_commands() {
        let commands =
            compile_script("@if cond=affection_eileen>=50\n# 艾琳\n走吧\n@endif\n# 旁白\n继续")
                .unwrap();

        assert!(matches!(
            &commands[0],
            Command::If {
                condition,
                then_commands,
                else_commands,
            } if condition == "affection_eileen>=50"
                && else_commands.is_empty()
                && matches!(&then_commands[0], Command::Text { speaker: Some(name), content } if name == "艾琳" && content == "走吧")
        ));
        assert!(matches!(
            &commands[1],
            Command::Text {
                speaker: Some(name),
                content
            } if name == "旁白" && content == "继续"
        ));
    }

    #[test]
    fn compiles_if_var_op_value_condition() {
        let commands =
            compile_script("@if var=score op=gt value=10\n@set name=passed value=true\n@endif")
                .unwrap();

        assert!(matches!(
            &commands[0],
            Command::If { condition, .. } if condition == "score>10"
        ));
    }

    #[test]
    fn compiles_if_else_block() {
        let commands = compile_script("@if cond=flag\n# N\n真\n@else\n# N\n假\n@endif").unwrap();

        assert!(matches!(
            &commands[0],
            Command::If {
                then_commands,
                else_commands,
                ..
            } if matches!(&then_commands[0], Command::Text { content, .. } if content == "真")
                && matches!(&else_commands[0], Command::Text { content, .. } if content == "假")
        ));
    }

    #[test]
    fn compiles_quoted_values_with_spaces() {
        let commands = compile_script(
            "@savename text=\"Chapter 1 - The First Bell\"\n@choice \"Go home\" goto=home",
        )
        .unwrap();

        assert!(matches!(
            &commands[0],
            Command::SaveName { text } if text == "Chapter 1 - The First Bell"
        ));
        assert!(matches!(
            &commands[1],
            Command::Choice { options } if options[0].text == "Go home"
        ));
    }

    #[test]
    fn registered_extension_commands_compile_as_custom_commands() {
        let mut registry = ExtensionRegistry::new();
        registry.register_command_name("shakeui");
        let document = parse_script("@shakeui \"dialogue\" power=8");

        let commands = compile_document_with_extensions(&document, Some(&registry)).unwrap();

        assert!(matches!(
            &commands[0],
            Command::Custom { name, args, attributes }
                if name == "shakeui"
                    && args == &vec!["dialogue".to_owned()]
                    && attributes[0].key == "power"
                    && attributes[0].value == "8"
        ));
    }
}
