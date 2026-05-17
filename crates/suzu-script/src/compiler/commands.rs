use suzu_core::{Color, Vec2};

use crate::{
    extension::ExtensionRegistry,
    parser::Attribute,
    vm::{
        Animation, AnimationKind, ChoiceOption, Command, CustomCommandAttribute, Position,
        Transition, VisualEffect,
    },
};

use super::{
    attributes::{
        optional, optional_bool, optional_color, optional_f32, optional_i32, optional_u32, required,
    },
    suggestions::suggest_command,
    CompileError,
};

pub(super) fn compile_command(
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
            options: vec![ChoiceOption {
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
