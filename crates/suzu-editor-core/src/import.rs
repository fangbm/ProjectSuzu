use std::path::PathBuf;

use suzu_core::Vec2;
use suzu_script::{
    parse_script,
    parser::{Attribute, SourceSpan},
    AstNode,
};

use crate::document::{
    AnimationForm, AudioForm, ChoiceOptionForm, CommandArgForm, EditorComment, EditorDocument,
    EditorNodeKind, EffectForm, PositionForm, TransitionForm,
};
use crate::graph::rebuild_edges;

pub fn import_szs(source: &str, source_path: Option<PathBuf>) -> EditorDocument {
    let parsed = parse_script(source);
    let mut doc = EditorDocument {
        source_path,
        ..EditorDocument::default()
    };
    let mut current_speaker = None::<String>;
    let mut index = 0;

    while index < parsed.nodes.len() {
        let span = parsed.spans.get(index).copied();
        match &parsed.nodes[index] {
            AstNode::Comment(text) => doc.comments.push(EditorComment {
                node: None,
                text: text.clone(),
            }),
            AstNode::Speaker(speaker) => current_speaker = Some(speaker.clone()),
            AstNode::Text(text) => {
                let mut merged = text.clone();
                let mut lookahead = index + 1;
                while matches!(parsed.nodes.get(lookahead), Some(AstNode::Text(_))) {
                    if let Some(AstNode::Text(next)) = parsed.nodes.get(lookahead) {
                        merged.push('\n');
                        merged.push_str(next);
                    }
                    lookahead += 1;
                }
                doc.push_node(
                    EditorNodeKind::Dialogue {
                        speaker: current_speaker.clone(),
                        text: merged,
                    },
                    span,
                );
                index = lookahead;
                continue;
            }
            AstNode::Label(name) => {
                doc.push_node(EditorNodeKind::Label { name: name.clone() }, span);
            }
            AstNode::Command {
                name,
                args,
                attributes,
            } if name == "choice" => {
                let mut options = Vec::new();
                let mut lookahead = index;
                while let Some(AstNode::Command {
                    name,
                    args,
                    attributes,
                }) = parsed.nodes.get(lookahead)
                {
                    if name != "choice" {
                        break;
                    }
                    options.push(ChoiceOptionForm {
                        text: args.first().cloned().unwrap_or_default(),
                        goto: attr(attributes, "goto").unwrap_or_default(),
                        condition: attr(attributes, "cond"),
                    });
                    lookahead += 1;
                }
                doc.push_node(EditorNodeKind::Choice { options }, span);
                index = lookahead;
                continue;
            }
            AstNode::Command {
                name,
                args,
                attributes,
            } => {
                if let Some(kind) = command_node(name, args, attributes, span, &mut doc) {
                    doc.push_node(kind, span);
                }
            }
        }
        index += 1;
    }

    rebuild_edges(&mut doc);
    doc
}

fn command_node(
    name: &str,
    args: &[String],
    attributes: &[Attribute],
    span: Option<SourceSpan>,
    doc: &mut EditorDocument,
) -> Option<EditorNodeKind> {
    match name {
        "script" => {
            let version = attr(attributes, "version")
                .and_then(|value| value.parse().ok())
                .unwrap_or(suzu_script::CURRENT_SCRIPT_FORMAT_VERSION);
            doc.metadata.script_version = version;
            Some(EditorNodeKind::ScriptHeader { version })
        }
        "bg" => Some(EditorNodeKind::Background {
            file: attr(attributes, "file").unwrap_or_default(),
            method: transition_form(attributes),
            time_ms: attr_u32(attributes, "time").unwrap_or(0),
        }),
        "char" => Some(EditorNodeKind::Character {
            name: attr(attributes, "name").unwrap_or_default(),
            face: attr(attributes, "face").filter(|face| !face.is_empty()),
            position: position_form(attributes),
            size: size_form(attributes),
            layer: attr_i32(attributes, "layer").unwrap_or(10),
            flip: attr_bool(attributes, "flip").unwrap_or(false),
        }),
        "hidechar" => Some(EditorNodeKind::HideCharacter {
            name: attr(attributes, "name").unwrap_or_default(),
        }),
        "anim" => Some(EditorNodeKind::Animation {
            target: attr(attributes, "target").unwrap_or_default(),
            form: animation_form(attributes),
        }),
        "fx" => Some(EditorNodeKind::Effect {
            form: effect_form(attributes),
        }),
        "set" => Some(EditorNodeKind::SetVariable {
            name: attr(attributes, "name").unwrap_or_default(),
            value: attr(attributes, "value").unwrap_or_default(),
        }),
        "jump" => Some(EditorNodeKind::Jump {
            label: attr(attributes, "goto").unwrap_or_default(),
        }),
        "call" => Some(EditorNodeKind::Call {
            label: attr(attributes, "goto").unwrap_or_default(),
        }),
        "return" => Some(EditorNodeKind::Return),
        "wait" => Some(EditorNodeKind::Wait {
            time_ms: attr_u32(attributes, "time").unwrap_or(0),
        }),
        "playbgm" => Some(EditorNodeKind::Audio {
            form: AudioForm::PlayBgm {
                file: attr(attributes, "file").unwrap_or_default(),
                looping: attr_bool(attributes, "loop").unwrap_or(true),
                fadein_ms: attr_u32(attributes, "fadein").unwrap_or(0),
            },
        }),
        "stopbgm" => Some(EditorNodeKind::Audio {
            form: AudioForm::StopBgm {
                fadeout_ms: attr_u32(attributes, "fadeout").unwrap_or(0),
            },
        }),
        "playvoice" => Some(EditorNodeKind::Audio {
            form: AudioForm::PlayVoice {
                file: attr(attributes, "file").unwrap_or_default(),
                fadein_ms: attr_u32(attributes, "fadein").unwrap_or(0),
            },
        }),
        "voice" => Some(EditorNodeKind::Audio {
            form: AudioForm::CueVoice {
                file: attr(attributes, "file").unwrap_or_default(),
                fadein_ms: attr_u32(attributes, "fadein").unwrap_or(0),
            },
        }),
        "stopvoice" => Some(EditorNodeKind::Audio {
            form: AudioForm::StopVoice {
                fadeout_ms: attr_u32(attributes, "fadeout").unwrap_or(0),
            },
        }),
        "hidemsg" => Some(EditorNodeKind::MessageBox { visible: false }),
        "showmsg" => Some(EditorNodeKind::MessageBox { visible: true }),
        "savename" => Some(EditorNodeKind::SaveName {
            text: attr(attributes, "text").unwrap_or_default(),
        }),
        "autosave" => Some(EditorNodeKind::AutoSave),
        "if" => Some(EditorNodeKind::If {
            condition: attr(attributes, "cond").unwrap_or_default(),
            then_nodes: Vec::new(),
            else_nodes: Vec::new(),
        }),
        "else" | "endif" => Some(EditorNodeKind::RawText {
            source: format!("@{name}"),
        }),
        _ => Some(EditorNodeKind::CustomCommand {
            name: name.to_owned(),
            args: args
                .iter()
                .cloned()
                .map(|value| CommandArgForm { key: None, value })
                .chain(attributes.iter().cloned().map(|attribute| CommandArgForm {
                    key: Some(attribute.key),
                    value: attribute.value,
                }))
                .collect(),
        }),
    }
    .or_else(|| {
        span.map(|_| EditorNodeKind::RawText {
            source: format!("@{name}"),
        })
    })
}

fn attr(attributes: &[Attribute], key: &str) -> Option<String> {
    attributes
        .iter()
        .find(|attribute| attribute.key == key)
        .map(|attribute| attribute.value.clone())
}

fn attr_u32(attributes: &[Attribute], key: &str) -> Option<u32> {
    attr(attributes, key).and_then(|value| value.parse().ok())
}

fn attr_i32(attributes: &[Attribute], key: &str) -> Option<i32> {
    attr(attributes, key).and_then(|value| value.parse().ok())
}

fn attr_bool(attributes: &[Attribute], key: &str) -> Option<bool> {
    attr(attributes, key).and_then(|value| value.parse().ok())
}

fn transition_form(attributes: &[Attribute]) -> TransitionForm {
    match attr(attributes, "method").as_deref() {
        Some("crossfade") => TransitionForm::CrossFade {
            duration_ms: attr_u32(attributes, "time").unwrap_or(0),
        },
        Some("fade") | Some("fade-through-color") => TransitionForm::FadeThroughColor {
            color: attr(attributes, "color").unwrap_or_else(|| "#000000".to_owned()),
            duration_ms: attr_u32(attributes, "time").unwrap_or(0),
        },
        _ => TransitionForm::Instant,
    }
}

fn position_form(attributes: &[Attribute]) -> PositionForm {
    match attr(attributes, "pos").as_deref() {
        Some("left") => PositionForm::Left,
        Some("right") => PositionForm::Right,
        Some("custom") => PositionForm::Custom {
            x: attr(attributes, "x")
                .and_then(|value| value.parse().ok())
                .unwrap_or(0.0),
            y: attr(attributes, "y")
                .and_then(|value| value.parse().ok())
                .unwrap_or(0.0),
        },
        _ => PositionForm::Center,
    }
}

fn size_form(attributes: &[Attribute]) -> Option<Vec2> {
    let width = attr(attributes, "w").and_then(|value| value.parse().ok())?;
    let height = attr(attributes, "h").and_then(|value| value.parse().ok())?;
    Some(Vec2::new(width, height))
}

fn animation_form(attributes: &[Attribute]) -> AnimationForm {
    let duration_ms = attr_u32(attributes, "time").unwrap_or(500);
    match attr(attributes, "type").as_deref() {
        Some("move") => AnimationForm::MoveTo {
            x: attr(attributes, "x")
                .and_then(|value| value.parse().ok())
                .unwrap_or(0.0),
            y: attr(attributes, "y")
                .and_then(|value| value.parse().ok())
                .unwrap_or(0.0),
            duration_ms,
        },
        Some("zoom") => AnimationForm::Zoom {
            center_x: attr(attributes, "x")
                .and_then(|value| value.parse().ok())
                .unwrap_or(0.0),
            center_y: attr(attributes, "y")
                .and_then(|value| value.parse().ok())
                .unwrap_or(0.0),
            scale: attr(attributes, "scale")
                .and_then(|value| value.parse().ok())
                .unwrap_or(1.0),
            duration_ms,
        },
        Some("fade") => AnimationForm::FadeTo {
            opacity: attr(attributes, "opacity")
                .and_then(|value| value.parse().ok())
                .unwrap_or(1.0),
            duration_ms,
        },
        _ => AnimationForm::Shake {
            intensity: attr(attributes, "intensity")
                .and_then(|value| value.parse().ok())
                .unwrap_or(8.0),
            duration_ms,
        },
    }
}

fn effect_form(attributes: &[Attribute]) -> EffectForm {
    let duration_ms = attr_u32(attributes, "time").unwrap_or(250);
    match attr(attributes, "kind").as_deref() {
        Some("quake") => EffectForm::Quake {
            intensity: attr(attributes, "intensity")
                .and_then(|value| value.parse().ok())
                .unwrap_or(8.0),
            duration_ms,
        },
        _ => EffectForm::Flash {
            color: attr(attributes, "color").unwrap_or_else(|| "#ffffff".to_owned()),
            duration_ms,
        },
    }
}
