use std::{
    collections::{HashMap, HashSet, VecDeque},
    fs,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context};
use encoding_rs::{SHIFT_JIS, UTF_16BE, UTF_16LE};
use suzu_asset::{probe_krkr_directory, KrkrCompatibilityReport, Xp3Archive, Xp3Options};
use suzu_editor_core::convert_krkr_ks_to_szs;
use suzu_script::CURRENT_SCRIPT_FORMAT_VERSION;

#[derive(Debug, Clone)]
pub struct KrkrConversionSummary {
    pub script_path: PathBuf,
    pub scripts: usize,
    pub unreadable: usize,
    pub lines: usize,
    pub commands: usize,
    pub choices: usize,
}

pub fn convert_krkr_package_to_suzu_project(
    root: &Path,
    output_root: &Path,
    option_candidates: &[Xp3Options],
) -> anyhow::Result<KrkrConversionSummary> {
    let compatibility = probe_krkr_directory(root).ok();
    let script_dir = output_root.join("script");
    fs::create_dir_all(&script_dir)
        .with_context(|| format!("failed to create {}", script_dir.display()))?;

    let mut archive_candidates = Vec::<Vec<Xp3Archive>>::new();
    let mut script_locations = HashMap::<String, (usize, String)>::new();
    let mut entrypoints = Vec::<String>::new();
    let mut fallback_scripts = Vec::<String>::new();

    for entry in fs::read_dir(root).with_context(|| format!("failed to scan {}", root.display()))? {
        let path = entry?.path();
        if !path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("xp3"))
        {
            continue;
        }
        let base_archive = match Xp3Archive::from_file_with_options(&path, Xp3Options::default()) {
            Ok(archive) => archive,
            Err(_) => continue,
        };
        let candidates = option_candidates
            .iter()
            .map(|options| base_archive.clone().with_options(options.clone()))
            .collect::<Vec<_>>();
        let Some(first_archive) = candidates.first() else {
            continue;
        };
        let archive_index = archive_candidates.len();
        let mut entries = first_archive
            .entries()
            .iter()
            .filter(|entry| script_extension_is(&entry.name, "ks"))
            .map(|entry| entry.name.clone())
            .collect::<Vec<_>>();
        entries.sort_by_key(|name| {
            (
                !krkr_entry_looks_like_entrypoint(name),
                name.to_ascii_lowercase(),
            )
        });

        for name in entries {
            for alias in krkr_script_lookup_keys(&name) {
                script_locations.insert(alias, (archive_index, name.clone()));
            }
            if krkr_entry_looks_like_entrypoint(&name) {
                entrypoints.push(name.clone());
            }
            fallback_scripts.push(name);
        }
        archive_candidates.push(candidates);
    }

    if script_locations.is_empty() {
        bail!("no .ks scripts found");
    }

    entrypoints.sort_by_key(|name| {
        (
            !krkr_entry_looks_like_entrypoint(name),
            name.to_ascii_lowercase(),
        )
    });
    let roots = if entrypoints.is_empty() {
        fallback_scripts.into_iter().take(1).collect::<Vec<_>>()
    } else {
        entrypoints
    };

    let mut scripts = Vec::<(String, Vec<u8>)>::new();
    let mut unreadable = 0usize;
    let mut queue = roots.into_iter().collect::<VecDeque<_>>();
    let mut visited = HashSet::<String>::new();
    while let Some(script_name) = queue.pop_front() {
        let lookup_key = normalize_krkr_script_key(&script_name);
        if !visited.insert(lookup_key.clone()) {
            continue;
        }
        if visited.len() > 512 {
            break;
        }

        let Some((archive_index, entry_name)) = script_locations.get(&lookup_key) else {
            continue;
        };
        let Some(bytes) = read_best_krkr_script(&archive_candidates[*archive_index], entry_name)
        else {
            unreadable += 1;
            continue;
        };

        let source = decode_krkr_text(&bytes);
        for reference in krkr_script_references(&source, entry_name) {
            if let Some((_, resolved_name)) =
                script_locations.get(&normalize_krkr_script_key(&reference))
            {
                queue.push_back(resolved_name.clone());
            }
        }
        scripts.push((entry_name.clone(), bytes));
    }

    if scripts.is_empty() {
        bail!("no readable .ks scripts found");
    }

    scripts.sort_by_key(|(name, _)| {
        (
            !krkr_entry_looks_like_entrypoint(name),
            name.to_ascii_lowercase(),
        )
    });
    scripts.dedup_by(|(left, _), (right, _)| left.eq_ignore_ascii_case(right));

    let mut output = format!(
        "@script version={CURRENT_SCRIPT_FORMAT_VERSION}\n; Converted from KRKR/KAG package: {}\n",
        root.display()
    );
    let mut total_lines = 0usize;
    let mut total_commands = 0usize;
    let mut total_choices = 0usize;
    for (entry_name, bytes) in &scripts {
        let source = decode_krkr_text(bytes);
        let converted = convert_krkr_ks_to_szs(&source, Some(entry_name));
        total_lines += converted.report.lines_read;
        total_commands += converted.report.commands_converted;
        total_choices += converted.report.choices;
        output.push('\n');
        for label in krkr_script_labels(entry_name) {
            output.push_str(&format!("*{label}\n"));
        }
        for line in converted.source.lines() {
            if line.trim_start().starts_with("@script ") {
                continue;
            }
            output.push_str(line);
            output.push('\n');
        }
    }

    if total_commands == 0
        && compatibility
            .as_ref()
            .is_some_and(KrkrCompatibilityReport::has_protected_entries)
    {
        let protected_scripts = compatibility
            .as_ref()
            .map_or(0, KrkrCompatibilityReport::protected_script_entries);
        bail!(
            "decoded KRKR scripts contain no KAG commands; this package has {protected_scripts} protected script-like entries. Run `suzu-launcher --krkr-probe <folder>` for details."
        );
    }

    suzu_script::compile_script(&output).context("converted KRKR startup flow did not compile")?;
    let script_path = script_dir.join("main.szs");
    fs::write(&script_path, output)
        .with_context(|| format!("failed to write {}", script_path.display()))?;

    Ok(KrkrConversionSummary {
        script_path,
        scripts: scripts.len(),
        unreadable,
        lines: total_lines,
        commands: total_commands,
        choices: total_choices,
    })
}

pub fn krkr_entry_looks_like_entrypoint(path: &str) -> bool {
    let normalized = path.replace('\\', "/").to_ascii_lowercase();
    matches!(
        normalized.as_str(),
        "startup.tjs"
            | "system/startup.tjs"
            | "appconfig.tjs"
            | "main/config.tjs"
            | "main/envinit.tjs"
            | "main/default.tjs"
            | "main/custom.ks"
            | "main/custom.tjs"
            | "first.ks"
            | "start.ks"
            | "title.ks"
    ) || normalized.ends_with("/startup.tjs")
        || normalized.ends_with("/first.ks")
        || normalized.ends_with("/start.ks")
        || normalized.ends_with("/title.ks")
}

fn script_extension_is(path: &str, extension: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case(extension))
}

fn read_best_krkr_script(archives: &[Xp3Archive], entry_name: &str) -> Option<Vec<u8>> {
    archives
        .iter()
        .filter_map(|archive| {
            let bytes = archive.read_file(entry_name).ok()?;
            let text = decode_krkr_text(&bytes);
            Some((score_krkr_text(&text), bytes))
        })
        .max_by_key(|(score, _)| *score)
        .map(|(_, bytes)| bytes)
}

fn score_krkr_text(text: &str) -> i64 {
    let mut score = 0i64;
    for ch in text.chars() {
        match ch {
            '[' | ']' | '@' | '*' | '=' | ';' | '"' | '\'' => score += 8,
            '\n' | '\r' | '\t' => score += 2,
            '\u{20}'..='\u{7e}' => score += 3,
            '\u{3040}'..='\u{30ff}' | '\u{3400}'..='\u{9fff}' => score += 2,
            '\u{0}'..='\u{8}' | '\u{b}' | '\u{c}' | '\u{e}'..='\u{1f}' => score -= 10,
            '\u{fffd}' => score -= 30,
            _ => score -= 1,
        }
    }
    score
}

fn krkr_script_references(source: &str, current_entry: &str) -> Vec<String> {
    let mut references = Vec::new();
    for token in source.split(|ch: char| ch.is_whitespace() || matches!(ch, '[' | ']')) {
        let Some((key, raw_value)) = token.split_once('=') else {
            continue;
        };
        if !key.eq_ignore_ascii_case("storage") {
            continue;
        }
        let value = raw_value
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
            .trim_matches(';');
        if !script_extension_is(value, "ks") {
            continue;
        }
        push_unique_reference(&mut references, value);
        if let Some(parent) = Path::new(current_entry).parent() {
            let relative = parent.join(value).to_string_lossy().replace('\\', "/");
            push_unique_reference(&mut references, &relative);
        }
    }
    references
}

fn push_unique_reference(references: &mut Vec<String>, value: &str) {
    if !value.is_empty()
        && !references
            .iter()
            .any(|existing| existing.eq_ignore_ascii_case(value))
    {
        references.push(value.to_owned());
    }
}

fn krkr_script_lookup_keys(path: &str) -> Vec<String> {
    let mut keys = Vec::new();
    push_unique_lookup_key(&mut keys, path);
    if let Some(file_name) = Path::new(path).file_name().and_then(|value| value.to_str()) {
        push_unique_lookup_key(&mut keys, file_name);
    }
    keys
}

fn push_unique_lookup_key(keys: &mut Vec<String>, value: &str) {
    let key = normalize_krkr_script_key(value);
    if !key.is_empty() && !keys.iter().any(|existing| existing == &key) {
        keys.push(key);
    }
}

fn normalize_krkr_script_key(path: &str) -> String {
    path.trim()
        .trim_matches('"')
        .trim_matches('\'')
        .replace('\\', "/")
        .to_ascii_lowercase()
}

fn krkr_script_labels(path: &str) -> Vec<String> {
    let mut labels = Vec::new();
    push_unique_label(&mut labels, path);
    if let Some(file_name) = Path::new(path).file_name().and_then(|value| value.to_str()) {
        push_unique_label(&mut labels, file_name);
    }
    if let Some(stem) = Path::new(path).file_stem().and_then(|value| value.to_str()) {
        push_unique_label(&mut labels, stem);
    }
    labels
}

fn push_unique_label(labels: &mut Vec<String>, raw: &str) {
    let label = sanitize_krkr_label(raw);
    if !label.is_empty() && !labels.iter().any(|existing| existing == &label) {
        labels.push(label);
    }
}

fn sanitize_krkr_label(label: &str) -> String {
    let label = label.trim().trim_start_matches('*');
    label
        .chars()
        .map(|ch| match ch {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '_' | '-' | '.' => ch,
            _ => '_',
        })
        .collect()
}

fn decode_krkr_text(bytes: &[u8]) -> String {
    if let Some(rest) = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]) {
        return String::from_utf8_lossy(rest).into_owned();
    }
    if let Some(rest) = bytes.strip_prefix(&[0xff, 0xfe]) {
        let (text, _, _) = UTF_16LE.decode(rest);
        return text.into_owned();
    }
    if let Some(rest) = bytes.strip_prefix(&[0xfe, 0xff]) {
        let (text, _, _) = UTF_16BE.decode(rest);
        return text.into_owned();
    }
    if let Ok(text) = std::str::from_utf8(bytes) {
        return text.to_owned();
    }
    let (text, _, _) = SHIFT_JIS.decode(bytes);
    text.into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_common_entrypoints() {
        assert!(krkr_entry_looks_like_entrypoint("main/default.tjs"));
        assert!(krkr_entry_looks_like_entrypoint("scenario/start.ks"));
        assert!(!krkr_entry_looks_like_entrypoint("scenario/extra.ks"));
    }

    #[test]
    fn script_references_include_relative_paths() {
        let references = krkr_script_references(r#"[jump storage="next.ks"]"#, "main/start.ks");
        assert_eq!(references, vec!["next.ks", "main/next.ks"]);
    }

    #[test]
    fn labels_are_sanitized_for_suzu_scripts() {
        assert_eq!(
            krkr_script_labels("main/start scene.ks"),
            vec!["main_start_scene.ks", "start_scene.ks", "start_scene"]
        );
    }
}
