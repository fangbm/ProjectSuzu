use super::*;

impl SuzuApp {
    pub(super) fn apply_command(&mut self, command_position: usize, command: Command) {
        match command {
            Command::Bg {
                file,
                time_ms,
                method,
            } => {
                self.ensure_frame_texture(&file);
                self.set_background(file, time_ms, method);
            }
            Command::Char {
                name,
                face,
                pos,
                size,
                flip_x,
                layer,
            } => {
                let texture_id = character_texture_id(&name, &face);
                self.ensure_frame_texture(&texture_id);
                let position = character_position(pos);
                upsert_character(
                    &mut self.scene.characters,
                    name,
                    texture_id,
                    position,
                    size,
                    flip_x,
                    layer,
                );
            }
            Command::HideChar { name } => {
                self.hide_character(&name);
            }
            Command::Text { speaker, content } => {
                let history_voice_file =
                    self.pending_voice.as_ref().map(|voice| voice.file.clone());
                self.current_dialogue_key = Some(dialogue_key(
                    &self.config.script_entry,
                    command_position,
                    speaker.as_deref(),
                    &content,
                ));
                let history_text = normalize_text_markup(&content);
                self.history.push(HistoryEntry {
                    speaker: speaker.clone(),
                    text: history_text,
                    voice_file: history_voice_file,
                });
                let mut block = TextBlock::plain(content, Rect::new(120.0, 500.0, 1040.0, 160.0));
                if let Some(speaker) = speaker {
                    let prefix = format!("{speaker}: ");
                    block.shift_wait_points(prefix.chars().count());
                    block.raw = format!("{prefix}{}", block.raw);
                    block.reveal.total_chars = block.raw.chars().count();
                }
                block.reveal.speed_chars_per_second =
                    self.settings.text.speed_chars_per_second.max(1.0);
                if let Some(voice) = self.pending_voice.take() {
                    block.segments.push(TextSegment::VoiceSync {
                        char_index: 0,
                        voice_file: voice.file.clone(),
                    });
                    self.play_voice(voice.file, voice.fadein_ms);
                }
                self.scene.dialogue = Some(block);
                self.scene.message_box_visible = true;
                self.auto_advance_elapsed_ms = 0;
            }
            Command::PlayBgm {
                file,
                looping,
                fadein_ms,
            } => {
                self.audio.bgm.play(
                    AudioSource::File {
                        path: file,
                        looping,
                    },
                    fadein_ms,
                );
            }
            Command::StopBgm { fadeout_ms } => {
                self.audio.bgm.stop(fadeout_ms);
            }
            Command::PlayVoice { file, fadein_ms } => {
                self.play_voice(file, fadein_ms);
            }
            Command::CueVoice { file, fadein_ms } => {
                self.pending_voice = Some(PendingVoice { file, fadein_ms });
            }
            Command::StopVoice { fadeout_ms } => {
                self.audio.voice.stop(fadeout_ms);
            }
            Command::Wait { duration_ms } => {
                self.wait_timer_ms = (duration_ms > 0).then_some(duration_ms);
            }
            Command::MessageBox { visible } => {
                self.scene.message_box_visible = visible;
            }
            Command::Anim { target, animation } => {
                self.start_animation(&target, animation.kind, animation.duration_ms)
            }
            Command::Choice { options } => {
                let options = options
                    .into_iter()
                    .filter(|option| {
                        option.condition.as_ref().map_or(true, |condition| {
                            evaluate_condition(condition, &self.variables)
                        })
                    })
                    .collect::<Vec<_>>();
                self.scene.choice = (!options.is_empty()).then(|| ChoiceState::new(options));
                self.auto_advance_elapsed_ms = 0;
                if self.scene.choice.is_some() {
                    self.skip_mode = false;
                }
            }
            Command::If {
                condition,
                then_commands,
                else_commands,
            } => {
                if evaluate_condition(&condition, &self.variables) {
                    self.script.insert_next(then_commands);
                } else {
                    self.script.insert_next(else_commands);
                }
            }
            Command::Jump { label } => {
                self.script.jump_to(&label);
            }
            Command::Call { label } => {
                self.script.call(&label);
            }
            Command::Return => {
                self.script.return_from_call();
            }
            Command::SetVar { name, value } => {
                self.variables.insert(name, parse_value(&value));
            }
            Command::Fx { effect } => self.start_visual_effect(effect),
            Command::Label { .. } => {}
            Command::SaveName { text } => {
                self.save_title = text;
            }
            Command::AutoSave => {
                let state = self.capture_state();
                self.saves.set_autosave(state);
            }
            Command::Custom { .. } => {}
        }
    }
}

fn parse_value(value: &str) -> Value {
    match value {
        "true" => Value::Bool(true),
        "false" => Value::Bool(false),
        _ => value
            .parse::<f64>()
            .map(Value::Number)
            .unwrap_or_else(|_| Value::Text(value.to_owned())),
    }
}

fn evaluate_condition(condition: &str, variables: &HashMap<String, Value>) -> bool {
    let condition = condition.trim();
    if let Some(name) = condition.strip_prefix('!') {
        return !truthy(variables.get(name.trim()));
    }

    for operator in ["==", "!=", ">=", "<=", ">", "<"] {
        if let Some((left, right)) = condition.split_once(operator) {
            return compare_values(
                variables.get(left.trim()),
                operator,
                &parse_value(right.trim().trim_matches('"')),
            );
        }
    }

    truthy(variables.get(condition))
}

fn compare_values(left: Option<&Value>, operator: &str, right: &Value) -> bool {
    let Some(left) = left else {
        return matches!(operator, "!=");
    };

    match operator {
        "==" => values_equal(left, right),
        "!=" => !values_equal(left, right),
        ">" => numeric_value(left)
            .zip(numeric_value(right))
            .is_some_and(|(l, r)| l > r),
        "<" => numeric_value(left)
            .zip(numeric_value(right))
            .is_some_and(|(l, r)| l < r),
        ">=" => numeric_value(left)
            .zip(numeric_value(right))
            .is_some_and(|(l, r)| l >= r),
        "<=" => numeric_value(left)
            .zip(numeric_value(right))
            .is_some_and(|(l, r)| l <= r),
        _ => false,
    }
}

fn values_equal(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Bool(left), Value::Bool(right)) => left == right,
        (Value::Number(left), Value::Number(right)) => (left - right).abs() < f64::EPSILON,
        (Value::Text(left), Value::Text(right)) => left == right,
        _ => false,
    }
}

fn numeric_value(value: &Value) -> Option<f64> {
    match value {
        Value::Number(value) => Some(*value),
        Value::Text(value) => value.parse().ok(),
        Value::Bool(_) => None,
    }
}

pub(super) fn wrapped_index(index: usize, len: usize, delta: i32) -> usize {
    if len == 0 {
        return 0;
    }

    if delta >= 0 {
        (index + delta as usize) % len
    } else {
        let delta = delta.unsigned_abs() as usize % len;
        (index + len - delta) % len
    }
}

fn truthy(value: Option<&Value>) -> bool {
    match value {
        Some(Value::Bool(value)) => *value,
        Some(Value::Number(value)) => *value != 0.0,
        Some(Value::Text(value)) => !value.is_empty(),
        None => false,
    }
}

fn dialogue_key(
    script_entry: &str,
    command_position: usize,
    speaker: Option<&str>,
    content: &str,
) -> String {
    let speaker = speaker.unwrap_or("");
    let text = normalize_text_markup(content);
    format!("{script_entry}:{command_position}:{speaker}:{text}")
}

pub(super) fn restored_dialogue_key(script_entry: &str, dialogue: &TextBlock) -> String {
    format!("{script_entry}:restored::{}", dialogue.raw)
}

pub(super) fn sorted_read_dialogue_keys(read_dialogue_keys: &HashSet<String>) -> Vec<String> {
    let mut keys = read_dialogue_keys.iter().cloned().collect::<Vec<_>>();
    keys.sort();
    keys
}
