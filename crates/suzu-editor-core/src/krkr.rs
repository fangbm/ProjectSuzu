use suzu_script::CURRENT_SCRIPT_FORMAT_VERSION;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KrkrConversion {
    pub source: String,
    pub report: KrkrConversionReport,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct KrkrConversionReport {
    pub lines_read: usize,
    pub labels: usize,
    pub text_lines: usize,
    pub commands_converted: usize,
    pub commands_preserved: usize,
    pub choices: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct KagTag {
    name: String,
    args: Vec<String>,
    attrs: Vec<(String, String)>,
}

pub fn convert_krkr_ks_to_szs(source: &str, source_name: Option<&str>) -> KrkrConversion {
    let mut converter = KrkrConverter {
        output: vec![format!("@script version={CURRENT_SCRIPT_FORMAT_VERSION}")],
        report: KrkrConversionReport::default(),
    };

    if let Some(source_name) = source_name.filter(|name| !name.trim().is_empty()) {
        converter
            .output
            .push(format!("; Converted from KRKR/KAG script: {source_name}"));
    } else {
        converter
            .output
            .push("; Converted from KRKR/KAG script".to_owned());
    }

    for raw_line in source.lines() {
        converter.report.lines_read += 1;
        converter.convert_line(raw_line);
    }

    let mut output = converter.output.join("\n");
    output.push('\n');
    KrkrConversion {
        source: output,
        report: converter.report,
    }
}

struct KrkrConverter {
    output: Vec<String>,
    report: KrkrConversionReport,
}

impl KrkrConverter {
    fn convert_line(&mut self, raw_line: &str) {
        let line = raw_line.trim();
        if line.is_empty() {
            return;
        }

        if let Some(comment) = line.strip_prefix(';') {
            self.output.push(format!(";{}", comment.trim_start()));
            return;
        }

        if let Some(label) = line.strip_prefix('*') {
            let label = label
                .split_once('|')
                .map_or(label, |(name, _)| name)
                .trim()
                .trim_start_matches('*');
            if !label.is_empty() {
                self.output.push(format!("*{}", sanitize_label(label)));
                self.report.labels += 1;
            }
            return;
        }

        if let Some(command) = line.strip_prefix('@') {
            if let Some(tag) = parse_kag_tag(command) {
                self.emit_tag(&tag);
            }
            return;
        }

        self.convert_mixed_text_line(line);
    }

    fn convert_mixed_text_line(&mut self, line: &str) {
        let mut text = String::new();
        let mut pending_choice: Option<(String, String)> = None;
        let chars = line.char_indices().collect::<Vec<_>>();
        let mut index = 0;

        while index < chars.len() {
            let (byte_index, ch) = chars[index];
            if ch != '[' {
                if let Some((_, choice_text)) = &mut pending_choice {
                    choice_text.push(ch);
                } else {
                    text.push(ch);
                }
                index += 1;
                continue;
            }

            let tag_start = byte_index + ch.len_utf8();
            let Some(close_offset) = line[tag_start..].find(']') else {
                if let Some((_, choice_text)) = &mut pending_choice {
                    choice_text.push(ch);
                } else {
                    text.push(ch);
                }
                index += 1;
                continue;
            };
            let tag_end = tag_start + close_offset;
            let tag_text = &line[tag_start..tag_end];
            let next_byte = tag_end + 1;
            index = chars
                .iter()
                .position(|(byte, _)| *byte >= next_byte)
                .unwrap_or(chars.len());

            let Some(tag) = parse_kag_tag(tag_text) else {
                continue;
            };

            match tag.name.as_str() {
                "link" => {
                    let target = attr(&tag, "target")
                        .or_else(|| attr(&tag, "storage"))
                        .map(clean_target)
                        .unwrap_or_default();
                    pending_choice = Some((target, String::new()));
                }
                "endlink" => {
                    if let Some((target, choice_text)) = pending_choice.take() {
                        self.flush_text(&mut text);
                        self.output.push(format!(
                            "@choice {} goto={}",
                            quote(choice_text.trim()),
                            quote(&target)
                        ));
                        self.report.choices += 1;
                    }
                }
                "r" | "l" => {
                    if pending_choice.is_none() {
                        self.flush_text(&mut text);
                    }
                }
                "p" | "s" => {
                    if pending_choice.is_none() {
                        self.flush_text(&mut text);
                    }
                }
                _ if pending_choice.is_some() => {}
                _ => {
                    self.flush_text(&mut text);
                    self.emit_tag(&tag);
                }
            }
        }

        if let Some((target, choice_text)) = pending_choice.take() {
            self.flush_text(&mut text);
            self.output.push(format!(
                "@choice {} goto={}",
                quote(choice_text.trim()),
                quote(&target)
            ));
            self.report.choices += 1;
        }
        self.flush_text(&mut text);
    }

    fn flush_text(&mut self, text: &mut String) {
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            self.output.push(trimmed.to_owned());
            self.report.text_lines += 1;
        }
        text.clear();
    }

    fn emit_tag(&mut self, tag: &KagTag) {
        let converted = match tag.name.as_str() {
            "bg" | "backlay" => attr(tag, "storage")
                .or_else(|| attr(tag, "file"))
                .map(|file| {
                    let time = attr(tag, "time").unwrap_or_else(|| "0".to_owned());
                    format!("@bg file={} method=crossfade time={time}", quote(&file))
                }),
            "image" | "ch" | "chara_show" | "char" => character_line(tag),
            "free" | "chara_hide" | "hidechar" => attr(tag, "name")
                .or_else(|| attr(tag, "storage"))
                .or_else(|| attr(tag, "layer"))
                .map(|name| format!("@hidechar name={}", quote(&name))),
            "playbgm" | "bgm" => attr(tag, "storage")
                .or_else(|| attr(tag, "file"))
                .map(|file| format!("@playbgm file={} loop=true fadein=0", quote(&file))),
            "stopbgm" => Some("@stopbgm fadeout=0".to_owned()),
            "playvoice" | "voice" | "voconfig" => attr(tag, "storage")
                .or_else(|| attr(tag, "file"))
                .map(|file| format!("@voice file={} fadein=0", quote(&file))),
            "wait" | "wt" => attr(tag, "time")
                .or_else(|| tag.args.first().cloned())
                .map(|time| format!("@wait time={}", numeric_or_zero(&time))),
            "jump" => attr(tag, "target")
                .or_else(|| attr(tag, "storage"))
                .map(|target| format!("@jump goto={}", quote(&clean_target(target)))),
            "call" => attr(tag, "target")
                .or_else(|| attr(tag, "storage"))
                .map(|target| format!("@call goto={}", quote(&clean_target(target)))),
            "return" => Some("@return".to_owned()),
            "select" | "button" => choice_from_tag(tag),
            "cm" | "ct" | "er" => Some("@showmsg".to_owned()),
            _ => None,
        };

        if let Some(line) = converted {
            self.output.push(line);
            self.report.commands_converted += 1;
        } else if !matches!(tag.name.as_str(), "trans" | "wt" | "l" | "r" | "p" | "s") {
            self.output
                .push(format!("; KRKR [{}]", reconstruct_tag(tag)));
            self.report.commands_preserved += 1;
        }
    }
}

fn character_line(tag: &KagTag) -> Option<String> {
    let file = attr(tag, "storage").or_else(|| attr(tag, "file"));
    let name = attr(tag, "name")
        .or_else(|| attr(tag, "id"))
        .or_else(|| file.clone())
        .or_else(|| attr(tag, "layer"))?;
    let mut parts = vec![format!("@char name={}", quote(&name))];
    if let Some(file) = file {
        parts.push(format!("face={}", quote(&file)));
    }
    if let Some(layer) = attr(tag, "layer").and_then(|value| value.parse::<i32>().ok()) {
        parts.push(format!("layer={layer}"));
    }
    if let Some(x) = attr(tag, "x") {
        parts.push(format!("x={}", numeric_or_zero(&x)));
    }
    if let Some(y) = attr(tag, "y") {
        parts.push(format!("y={}", numeric_or_zero(&y)));
    }
    Some(parts.join(" "))
}

fn choice_from_tag(tag: &KagTag) -> Option<String> {
    let text = attr(tag, "text")
        .or_else(|| attr(tag, "caption"))
        .or_else(|| tag.args.first().cloned())?;
    let target = attr(tag, "target")
        .or_else(|| attr(tag, "storage"))
        .map(clean_target)
        .unwrap_or_default();
    Some(format!("@choice {} goto={}", quote(&text), quote(&target)))
}

fn parse_kag_tag(source: &str) -> Option<KagTag> {
    let tokens = tokenize(source);
    let mut parts = tokens.into_iter();
    let name = parts.next()?.trim_start_matches('@').to_ascii_lowercase();
    if name.is_empty() {
        return None;
    }

    let mut args = Vec::new();
    let mut attrs = Vec::new();
    for token in parts {
        if let Some((key, value)) = token.split_once('=') {
            attrs.push((
                key.trim().to_ascii_lowercase(),
                value.trim().trim_matches('"').trim_matches('\'').to_owned(),
            ));
        } else {
            args.push(token);
        }
    }

    Some(KagTag { name, args, attrs })
}

fn tokenize(source: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut chars = source.chars().peekable();
    let mut quote = None::<char>;

    while let Some(ch) = chars.next() {
        match (ch, quote) {
            ('"' | '\'', None) => quote = Some(ch),
            (value, Some(active)) if value == active => quote = None,
            ('\\', Some(_)) => {
                if let Some(next) = chars.next() {
                    current.push(next);
                }
            }
            (';' | '#', None) => break,
            (ch, None) if ch.is_whitespace() => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
                while chars.peek().is_some_and(|next| next.is_whitespace()) {
                    chars.next();
                }
            }
            _ => current.push(ch),
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

fn attr(tag: &KagTag, key: &str) -> Option<String> {
    tag.attrs
        .iter()
        .find(|(candidate, _)| candidate == key)
        .map(|(_, value)| value.clone())
}

fn clean_target(target: impl AsRef<str>) -> String {
    let target = target.as_ref().trim();
    let target = target.strip_prefix('*').unwrap_or(target);
    sanitize_label(target)
}

fn sanitize_label(label: &str) -> String {
    label
        .chars()
        .map(|ch| match ch {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '_' | '-' | '.' => ch,
            _ => '_',
        })
        .collect()
}

fn numeric_or_zero(value: &str) -> &str {
    if value.parse::<u32>().is_ok() {
        value
    } else {
        "0"
    }
}

fn quote(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

fn reconstruct_tag(tag: &KagTag) -> String {
    let mut parts = vec![tag.name.clone()];
    parts.extend(tag.args.iter().cloned());
    parts.extend(
        tag.attrs
            .iter()
            .map(|(key, value)| format!("{key}={}", quote(value))),
    );
    parts.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_basic_kag_script_to_compilable_szs() {
        let converted = convert_krkr_ks_to_szs(
            r#"; sample
*start|Start
[bg storage="school.png" time=500]
[playbgm storage="theme.ogg"]
# Alice
Hello[l][r]
[link target=*yes]Yes[endlink]
[jump target=*end]
*yes
[voice storage="alice001.ogg"]Good.
*end
[stopbgm]"#,
            Some("main.ks"),
        );

        assert!(converted.source.contains("*start"));
        assert!(converted.source.contains("@bg file=\"school.png\""));
        assert!(converted.source.contains("@choice \"Yes\" goto=\"yes\""));
        assert!(converted.source.contains("@voice file=\"alice001.ogg\""));
        suzu_script::compile_script(&converted.source).unwrap();
    }

    #[test]
    fn preserves_unknown_tags_as_comments() {
        let converted = convert_krkr_ks_to_szs("[quake time=200]\nText", None);

        assert!(converted.source.contains("; KRKR [quake time=\"200\"]"));
        assert!(converted.source.contains("Text"));
        suzu_script::compile_script(&converted.source).unwrap();
    }
}
