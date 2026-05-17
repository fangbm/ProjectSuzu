use super::{
    document::{command_node, parse_syntax_name, span_for_offset, unquote},
    AstNode, Attribute, ScriptDocument, ScriptSyntax, SourceSpan,
};

pub(super) fn parse_markup_script(source: &str, syntax: ScriptSyntax) -> ScriptDocument {
    let mut nodes = Vec::new();
    let mut spans = Vec::new();
    let mut cursor = 0;

    while let Some(relative_open) = source[cursor..].find('<') {
        let open = cursor + relative_open;
        flush_text(&mut nodes, &mut spans, source, cursor, open);

        let Some(relative_close) = source[open..].find('>') else {
            break;
        };
        let close = open + relative_close;
        parse_tag(
            &mut nodes,
            &mut spans,
            source,
            open,
            &source[open + 1..close],
        );
        cursor = close + 1;
    }

    flush_text(&mut nodes, &mut spans, source, cursor, source.len());

    ScriptDocument {
        syntax,
        nodes,
        spans,
    }
}

pub(super) fn syntax_from_script_tag(line: &str) -> Option<ScriptSyntax> {
    let open = line.find("<script")?;
    let close = line[open..].find('>')? + open;
    let tag = line[open + 1..close].trim().trim_end_matches('/');
    let (_, attrs) = split_tag(tag);
    find_attr(&attrs, "syntax")
        .or_else(|| find_attr(&attrs, "style"))
        .and_then(|value| parse_syntax_name(&value))
}

fn flush_text(
    nodes: &mut Vec<AstNode>,
    spans: &mut Vec<SourceSpan>,
    source: &str,
    start: usize,
    end: usize,
) {
    for (line_offset, line) in source[start..end].lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let span = span_for_offset(source, start + line_offset);
        if let Some(command) = line.strip_prefix('@') {
            nodes.push(super::document::parse_command(command));
            spans.push(span);
        } else {
            nodes.push(AstNode::Text(line.to_owned()));
            spans.push(span);
        }
    }
}

fn parse_tag(
    nodes: &mut Vec<AstNode>,
    spans: &mut Vec<SourceSpan>,
    source: &str,
    offset: usize,
    tag: &str,
) {
    let span = span_for_offset(source, offset);
    let tag = tag.trim();
    if tag.is_empty() || tag.starts_with('!') {
        return;
    }

    if let Some(closing) = tag.strip_prefix('/') {
        if closing.split_whitespace().next() == Some("if") {
            nodes.push(command_node("endif", Vec::new(), Vec::new()));
            spans.push(span);
        }
        return;
    }

    let tag = tag.trim_end_matches('/').trim();
    let (name, mut attributes) = split_tag(tag);
    match name.as_str() {
        "scene" | "route" | "body" => {}
        "script" => push_command(nodes, spans, span, "script", Vec::new(), attributes),
        "label" => {
            nodes.push(AstNode::Label(
                take_attr(&mut attributes, "name")
                    .or_else(|| take_attr(&mut attributes, "id"))
                    .unwrap_or_default(),
            ));
            spans.push(span);
        }
        "say" => {
            if let Some(speaker) = take_attr(&mut attributes, "speaker") {
                nodes.push(AstNode::Speaker(speaker));
                spans.push(span);
            }
            if let Some(text) = take_attr(&mut attributes, "text") {
                nodes.push(AstNode::Text(text));
                spans.push(span);
            }
        }
        "choice" => {
            let mut args = Vec::new();
            if let Some(text) = take_attr(&mut attributes, "text") {
                args.push(text);
            }
            push_command(nodes, spans, span, "choice", args, attributes);
        }
        "else" => push_command(nodes, spans, span, "else", Vec::new(), Vec::new()),
        "if" => push_command(nodes, spans, span, "if", Vec::new(), attributes),
        "comment" => {
            nodes.push(AstNode::Comment(
                take_attr(&mut attributes, "text").unwrap_or_default(),
            ));
            spans.push(span);
        }
        other => push_command(nodes, spans, span, other, Vec::new(), attributes),
    }
}

fn push_command(
    nodes: &mut Vec<AstNode>,
    spans: &mut Vec<SourceSpan>,
    span: SourceSpan,
    name: impl Into<String>,
    args: Vec<String>,
    attributes: Vec<Attribute>,
) {
    nodes.push(command_node(name, args, attributes));
    spans.push(span);
}

fn split_tag(tag: &str) -> (String, Vec<Attribute>) {
    let (name, rest) = tag
        .split_once(char::is_whitespace)
        .map_or((tag, ""), |(name, rest)| (name, rest));
    (name.to_owned(), parse_tag_attributes(rest))
}

fn parse_tag_attributes(input: &str) -> Vec<Attribute> {
    let mut attributes = Vec::new();
    let mut index = 0;
    let bytes = input.as_bytes();

    while index < input.len() {
        while index < input.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        let key_start = index;
        while index < input.len() && !bytes[index].is_ascii_whitespace() && bytes[index] != b'=' {
            index += 1;
        }
        if key_start == index {
            break;
        }
        let key = input[key_start..index].trim();
        while index < input.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        if index >= input.len() || bytes[index] != b'=' {
            continue;
        }
        index += 1;
        while index < input.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }

        let value = if index < input.len() && bytes[index] == b'"' {
            let value_start = index;
            index += 1;
            while index < input.len() && bytes[index] != b'"' {
                index += 1;
            }
            if index < input.len() {
                index += 1;
            }
            unquote(&input[value_start..index])
        } else {
            let value_start = index;
            while index < input.len() && !bytes[index].is_ascii_whitespace() {
                index += 1;
            }
            input[value_start..index].to_owned()
        };

        attributes.push(Attribute {
            key: key.to_owned(),
            value,
        });
    }

    attributes
}

fn find_attr(attributes: &[Attribute], key: &str) -> Option<String> {
    attributes
        .iter()
        .find(|attribute| attribute.key == key)
        .map(|attribute| attribute.value.clone())
}

fn take_attr(attributes: &mut Vec<Attribute>, key: &str) -> Option<String> {
    let index = attributes
        .iter()
        .position(|attribute| attribute.key == key)?;
    Some(attributes.remove(index).value)
}
