use super::{braces, indent, markup, AstNode, Attribute, ScriptDocument, ScriptSyntax, SourceSpan};

const COMMAND_NAMES: &[&str] = &[
    "script",
    "bg",
    "char",
    "hidechar",
    "hide",
    "jump",
    "call",
    "return",
    "set",
    "var",
    "savename",
    "autosave",
    "choice",
    "playbgm",
    "stopbgm",
    "playvoice",
    "voice",
    "stopvoice",
    "wait",
    "hidemsg",
    "hidemessage",
    "showmsg",
    "showmessage",
    "anim",
    "fx",
    "if",
    "else",
    "endif",
];

pub fn parse_script(source: &str) -> ScriptDocument {
    parse_script_with_syntax(source, detect_script_syntax(source))
}

pub fn parse_script_with_syntax(source: &str, syntax: ScriptSyntax) -> ScriptDocument {
    match syntax {
        ScriptSyntax::Classic => parse_classic_script(source, syntax),
        ScriptSyntax::Indent => indent::parse_indent_script(source, syntax),
        ScriptSyntax::Braces => braces::parse_braces_script(source, syntax),
        ScriptSyntax::Markup => markup::parse_markup_script(source, syntax),
    }
}

pub fn detect_script_syntax(source: &str) -> ScriptSyntax {
    for raw_line in source.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with(';') {
            continue;
        }

        if let Some(header) = line
            .strip_prefix("@script")
            .or_else(|| line.strip_prefix("script"))
        {
            return syntax_from_header_assignment(line)
                .or_else(|| syntax_from_attributes(parse_attribute_tokens(header)))
                .unwrap_or(ScriptSyntax::Classic);
        }

        if line.starts_with("<script") {
            return markup::syntax_from_script_tag(line).unwrap_or(ScriptSyntax::Markup);
        }

        break;
    }

    ScriptSyntax::Classic
}

fn syntax_from_header_assignment(line: &str) -> Option<ScriptSyntax> {
    ["syntax", "style"].into_iter().find_map(|key| {
        let index = line.find(key)?;
        let value = line[index + key.len()..].trim_start();
        let value = value.strip_prefix('=')?.trim_start();
        let value = value.trim_start_matches('"');
        let value = value
            .split(|ch: char| {
                ch.is_whitespace() || matches!(ch, '"' | '\'' | ',' | ')' | ';' | '>')
            })
            .next()
            .unwrap_or_default();
        parse_syntax_name(value)
    })
}

pub(super) fn parse_classic_script(source: &str, syntax: ScriptSyntax) -> ScriptDocument {
    let mut nodes = Vec::new();
    let mut spans = Vec::new();

    for (line_index, raw_line) in source.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        push_classic_line(
            &mut nodes,
            &mut spans,
            line,
            span_for_line(line_index, raw_line),
        );
    }

    ScriptDocument {
        syntax,
        nodes,
        spans,
    }
}

pub(super) fn push_classic_line(
    nodes: &mut Vec<AstNode>,
    spans: &mut Vec<SourceSpan>,
    line: &str,
    span: SourceSpan,
) {
    spans.push(span);

    if let Some(comment) = line.strip_prefix(';') {
        nodes.push(AstNode::Comment(comment.trim().to_owned()));
    } else if let Some(label) = line.strip_prefix('*') {
        nodes.push(AstNode::Label(label.trim().to_owned()));
    } else if let Some(speaker) = line.strip_prefix('#') {
        nodes.push(AstNode::Speaker(speaker.trim().to_owned()));
    } else if let Some(command) = line.strip_prefix('@') {
        nodes.push(parse_command(command));
    } else {
        nodes.push(AstNode::Text(line.to_owned()));
    }
}

pub(super) fn parse_command(command: &str) -> AstNode {
    let tokens = tokenize_command(command);
    command_from_tokens(tokens)
}

pub(super) fn command_from_tokens(tokens: Vec<String>) -> AstNode {
    let mut parts = tokens.into_iter();
    let name = parts.next().unwrap_or_default();
    let mut args = Vec::new();
    let mut attributes = Vec::new();

    for part in parts {
        if let Some((key, value)) = part.split_once('=') {
            attributes.push(Attribute {
                key: key.to_owned(),
                value: value.to_owned(),
            });
        } else {
            args.push(part);
        }
    }

    AstNode::Command {
        name,
        args,
        attributes,
    }
}

pub(super) fn command_node(
    name: impl Into<String>,
    args: Vec<String>,
    attributes: Vec<Attribute>,
) -> AstNode {
    AstNode::Command {
        name: name.into(),
        args,
        attributes,
    }
}

pub(super) fn tokenize_command(command: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut chars = command.chars().peekable();
    let mut in_quote = false;
    let mut escaped = false;

    while let Some(ch) = chars.next() {
        if escaped {
            current.push(ch);
            escaped = false;
            continue;
        }

        match ch {
            '\\' if in_quote => escaped = true,
            '"' => in_quote = !in_quote,
            ';' if !in_quote => break,
            ch if ch.is_whitespace() && !in_quote => {
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

pub(super) fn parse_attribute_tokens(value: &str) -> Vec<(String, String)> {
    tokenize_command(value)
        .into_iter()
        .filter_map(|token| {
            let (key, value) = token.split_once('=')?;
            Some((key.to_owned(), value.to_owned()))
        })
        .collect()
}

pub(super) fn syntax_from_attributes(
    attributes: impl IntoIterator<Item = (String, String)>,
) -> Option<ScriptSyntax> {
    attributes
        .into_iter()
        .filter(|(key, _)| *key == "syntax" || *key == "style")
        .find_map(|(_, value)| parse_syntax_name(&value))
}

pub(super) fn parse_syntax_name(value: &str) -> Option<ScriptSyntax> {
    match value.trim().to_ascii_lowercase().as_str() {
        "classic" | "suzu" | "szs" => Some(ScriptSyntax::Classic),
        "indent" | "indented" | "python" | "py" => Some(ScriptSyntax::Indent),
        "braces" | "brace" | "c" | "c_like" | "c-like" => Some(ScriptSyntax::Braces),
        "markup" | "html" | "xml" | "tag" | "tags" => Some(ScriptSyntax::Markup),
        _ => None,
    }
}

pub(super) fn span_for_line(line_index: usize, raw_line: &str) -> SourceSpan {
    SourceSpan {
        line: line_index + 1,
        column: raw_line
            .char_indices()
            .find(|(_, ch)| !ch.is_whitespace())
            .map_or(1, |(index, _)| index + 1),
    }
}

pub(super) fn span_for_offset(source: &str, offset: usize) -> SourceSpan {
    let mut line = 1;
    let mut column = 1;

    for (index, ch) in source.char_indices() {
        if index >= offset {
            break;
        }

        if ch == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }

    SourceSpan { line, column }
}

pub(super) fn indentation(raw_line: &str) -> usize {
    raw_line
        .chars()
        .take_while(|ch| ch.is_whitespace() && *ch != '\n')
        .map(|ch| if ch == '\t' { 4 } else { 1 })
        .sum()
}

pub(super) fn first_word(value: &str) -> &str {
    value
        .split_once(char::is_whitespace)
        .map_or(value, |(word, _)| word)
        .trim_matches(|ch: char| ch == '(' || ch == ':' || ch == ';')
}

pub(super) fn is_command_name(name: &str) -> bool {
    COMMAND_NAMES.contains(&name)
}

pub(super) fn unquote(value: &str) -> String {
    let value = value.trim();
    if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
        value[1..value.len() - 1].replace("\\\"", "\"")
    } else {
        value.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_command_and_text() {
        let doc = parse_script("@bg file=\"school\"\n# 艾琳\n你好");
        assert_eq!(doc.syntax, ScriptSyntax::Classic);
        assert_eq!(doc.nodes.len(), 3);
        assert!(matches!(&doc.nodes[0], AstNode::Command { name, .. } if name == "bg"));
    }

    #[test]
    fn detects_script_syntax_from_header() {
        let doc = parse_script("@script version=1 syntax=indent\nSuzu: Hi");
        assert_eq!(doc.syntax, ScriptSyntax::Indent);
    }

    #[test]
    fn detects_brace_syntax_from_call_style_header() {
        let doc = parse_script("script(version=1, syntax=braces);\nSuzu: Hi;");
        assert_eq!(doc.syntax, ScriptSyntax::Braces);
    }

    #[test]
    fn parses_quoted_arguments_with_spaces() {
        let doc = parse_script("@choice \"Go home\" goto=home cond=\"route name==home\"");

        let AstNode::Command {
            args, attributes, ..
        } = &doc.nodes[0]
        else {
            panic!("expected command");
        };
        assert_eq!(args, &["Go home"]);
        assert_eq!(attributes[0].value, "home");
        assert_eq!(attributes[1].value, "route name==home");
    }

    #[test]
    fn ignores_inline_comments_outside_quotes() {
        let doc = parse_script("@savename text=\"Chapter ; One\" ; visible comment");

        let AstNode::Command { attributes, .. } = &doc.nodes[0] else {
            panic!("expected command");
        };
        assert_eq!(attributes[0].value, "Chapter ; One");
    }
}
