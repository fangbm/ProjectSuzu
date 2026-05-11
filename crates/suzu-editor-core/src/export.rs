use anyhow::{Context, Result};

use crate::document::{
    AnimationForm, AudioForm, ChoiceOptionForm, CommandArgForm, EditorDocument, EditorNodeKind,
    EffectForm, PositionForm, TransitionForm,
};

pub fn export_szs(document: &EditorDocument) -> Result<String> {
    let mut lines = Vec::new();
    let mut last_speaker = None::<String>;
    let mut wrote_header = false;

    for node in &document.nodes {
        match &node.kind {
            EditorNodeKind::ScriptHeader { version } => {
                lines.push(format!("@script version={version}"));
                wrote_header = true;
            }
            EditorNodeKind::Label { name } => {
                lines.push(format!("*{name}"));
                last_speaker = None;
            }
            EditorNodeKind::Dialogue { speaker, text } => {
                if speaker != &last_speaker {
                    if let Some(speaker) = speaker {
                        lines.push(format!("# {speaker}"));
                    }
                    last_speaker = speaker.clone();
                }
                lines.extend(text.lines().map(str::to_owned));
            }
            EditorNodeKind::Background {
                file,
                method,
                time_ms,
            } => lines.push(background_line(file, method, *time_ms)),
            EditorNodeKind::Character {
                name,
                face,
                position,
                size,
                layer,
                flip,
            } => lines.push(character_line(
                name,
                face.as_deref(),
                position,
                *size,
                *layer,
                *flip,
            )),
            EditorNodeKind::HideCharacter { name } => {
                lines.push(format!("@hidechar name={}", quote(name)));
            }
            EditorNodeKind::Animation { target, form } => {
                lines.push(animation_line(target, form));
            }
            EditorNodeKind::Effect { form } => lines.push(effect_line(form)),
            EditorNodeKind::Choice { options } => {
                lines.extend(options.iter().map(choice_line));
            }
            EditorNodeKind::SetVariable { name, value } => {
                lines.push(format!("@set name={} value={}", quote(name), quote(value)));
            }
            EditorNodeKind::If { condition, .. } => {
                lines.push(format!("@if cond={}", quote(condition)));
                lines.push("@endif".to_owned());
            }
            EditorNodeKind::Jump { label } => lines.push(format!("@jump goto={}", quote(label))),
            EditorNodeKind::Call { label } => lines.push(format!("@call goto={}", quote(label))),
            EditorNodeKind::Return => lines.push("@return".to_owned()),
            EditorNodeKind::Wait { time_ms } => lines.push(format!("@wait time={time_ms}")),
            EditorNodeKind::Audio { form } => lines.push(audio_line(form)),
            EditorNodeKind::MessageBox { visible } => {
                lines.push(if *visible { "@showmsg" } else { "@hidemsg" }.to_owned());
            }
            EditorNodeKind::SaveName { text } => {
                lines.push(format!("@savename text={}", quote(text)));
            }
            EditorNodeKind::AutoSave => lines.push("@autosave".to_owned()),
            EditorNodeKind::CustomCommand { name, args } => {
                lines.push(custom_command_line(name, args));
            }
            EditorNodeKind::RawText { source } => lines.push(source.clone()),
        }
    }

    if !wrote_header {
        lines.insert(
            0,
            format!("@script version={}", document.metadata.script_version),
        );
    }

    let output = lines.join("\n");
    suzu_script::compile_script(&output).with_context(|| "exported script did not compile")?;
    Ok(format!("{output}\n"))
}

fn background_line(file: &str, method: &TransitionForm, time_ms: u32) -> String {
    match method {
        TransitionForm::Instant => {
            format!("@bg file={} method=instant time={time_ms}", quote(file))
        }
        TransitionForm::CrossFade { duration_ms } => format!(
            "@bg file={} method=crossfade time={}",
            quote(file),
            duration_ms
        ),
        TransitionForm::FadeThroughColor { color, duration_ms } => format!(
            "@bg file={} method=fade color={} time={}",
            quote(file),
            quote(color),
            duration_ms
        ),
    }
}

fn character_line(
    name: &str,
    face: Option<&str>,
    position: &PositionForm,
    size: Option<suzu_core::Vec2>,
    layer: i32,
    flip: bool,
) -> String {
    let mut parts = vec![format!("@char name={}", quote(name))];
    if let Some(face) = face.filter(|face| !face.is_empty()) {
        parts.push(format!("face={}", quote(face)));
    }
    match position {
        PositionForm::Left => parts.push("pos=left".to_owned()),
        PositionForm::Center => parts.push("pos=center".to_owned()),
        PositionForm::Right => parts.push("pos=right".to_owned()),
        PositionForm::Custom { x, y } => {
            parts.push("pos=custom".to_owned());
            parts.push(format!("x={x}"));
            parts.push(format!("y={y}"));
        }
    }
    if let Some(size) = size {
        parts.push(format!("w={}", size.x));
        parts.push(format!("h={}", size.y));
    }
    parts.push(format!("layer={layer}"));
    parts.push(format!("flip={flip}"));
    parts.join(" ")
}

fn animation_line(target: &str, form: &AnimationForm) -> String {
    match form {
        AnimationForm::Shake {
            intensity,
            duration_ms,
        } => format!(
            "@anim target={} type=shake intensity={} time={}",
            quote(target),
            intensity,
            duration_ms
        ),
        AnimationForm::MoveTo { x, y, duration_ms } => format!(
            "@anim target={} type=move x={} y={} time={}",
            quote(target),
            x,
            y,
            duration_ms
        ),
        AnimationForm::Zoom {
            center_x,
            center_y,
            scale,
            duration_ms,
        } => format!(
            "@anim target={} type=zoom x={} y={} scale={} time={}",
            quote(target),
            center_x,
            center_y,
            scale,
            duration_ms
        ),
        AnimationForm::FadeTo {
            opacity,
            duration_ms,
        } => format!(
            "@anim target={} type=fade opacity={} time={}",
            quote(target),
            opacity,
            duration_ms
        ),
    }
}

fn effect_line(form: &EffectForm) -> String {
    match form {
        EffectForm::Flash { color, duration_ms } => {
            format!("@fx kind=flash color={} time={duration_ms}", quote(color))
        }
        EffectForm::Quake {
            intensity,
            duration_ms,
        } => format!("@fx kind=quake intensity={intensity} time={duration_ms}"),
    }
}

fn choice_line(option: &ChoiceOptionForm) -> String {
    let mut line = format!(
        "@choice {} goto={}",
        quote(&option.text),
        quote(&option.goto)
    );
    if let Some(condition) = &option.condition {
        if !condition.is_empty() {
            line.push_str(&format!(" cond={}", quote(condition)));
        }
    }
    line
}

fn audio_line(form: &AudioForm) -> String {
    match form {
        AudioForm::PlayBgm {
            file,
            looping,
            fadein_ms,
        } => format!(
            "@playbgm file={} loop={} fadein={}",
            quote(file),
            looping,
            fadein_ms
        ),
        AudioForm::StopBgm { fadeout_ms } => format!("@stopbgm fadeout={fadeout_ms}"),
        AudioForm::PlayVoice { file, fadein_ms } => {
            format!("@playvoice file={} fadein={fadein_ms}", quote(file))
        }
        AudioForm::CueVoice { file, fadein_ms } => {
            format!("@voice file={} fadein={fadein_ms}", quote(file))
        }
        AudioForm::StopVoice { fadeout_ms } => format!("@stopvoice fadeout={fadeout_ms}"),
    }
}

fn custom_command_line(name: &str, args: &[CommandArgForm]) -> String {
    let mut parts = vec![format!("@{name}")];
    for arg in args {
        match &arg.key {
            Some(key) => parts.push(format!("{key}={}", quote(&arg.value))),
            None => parts.push(quote(&arg.value)),
        }
    }
    parts.join(" ")
}

fn quote(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

#[cfg(test)]
mod tests {
    use crate::import_szs;

    use super::*;

    #[test]
    fn exports_imported_dialogue_script() {
        let doc = import_szs("@script version=1\n# Eileen\nHello", None);
        let exported = export_szs(&doc).unwrap();

        assert!(exported.contains("# Eileen"));
        assert!(exported.contains("Hello"));
        suzu_script::compile_script(&exported).unwrap();
    }

    #[test]
    fn exports_choice_group_as_adjacent_choice_commands() {
        let doc = import_szs(
            "@choice \"A\" goto=a\n@choice \"B\" goto=b\n*a\nA\n*b\nB",
            None,
        );
        let exported = export_szs(&doc).unwrap();

        assert!(exported.contains("@choice \"A\" goto=\"a\"\n@choice \"B\" goto=\"b\""));
        suzu_script::compile_script(&exported).unwrap();
    }
}
