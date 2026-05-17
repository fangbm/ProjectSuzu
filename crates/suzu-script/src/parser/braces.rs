use super::{
    document::{
        command_node, first_word, is_command_name, parse_command, span_for_offset, unquote,
    },
    AstNode, Attribute, ScriptDocument, ScriptSyntax, SourceSpan,
};

pub(super) fn parse_braces_script(source: &str, syntax: ScriptSyntax) -> ScriptDocument {
    let mut nodes = Vec::new();
    let mut spans = Vec::new();

    let statements = split_statements(source);
    let mut index = 0;
    while let Some((statement, span)) = statements.get(index) {
        if statement == "}"
            && statements
                .get(index + 1)
                .is_some_and(|(next, _)| next == "else")
        {
            nodes.push(command_node("else", Vec::new(), Vec::new()));
            spans.push(statements[index + 1].1);
            index += 2;
            continue;
        }

        for node in parse_statement(statement) {
            nodes.push(node);
            spans.push(*span);
        }
        index += 1;
    }

    ScriptDocument {
        syntax,
        nodes,
        spans,
    }
}

fn split_statements(source: &str) -> Vec<(String, SourceSpan)> {
    let mut statements = Vec::new();
    let mut current = String::new();
    let mut start_offset = 0;
    let mut in_quote = false;
    let mut escaped = false;

    for (offset, ch) in source.char_indices() {
        if current.is_empty() && !ch.is_whitespace() {
            start_offset = offset;
        }

        if escaped {
            current.push(ch);
            escaped = false;
            continue;
        }

        match ch {
            '\\' if in_quote => {
                current.push(ch);
                escaped = true;
            }
            '"' => {
                in_quote = !in_quote;
                current.push(ch);
            }
            ';' if !in_quote => flush(source, &mut statements, &mut current, start_offset),
            '{' if !in_quote => flush(source, &mut statements, &mut current, start_offset),
            '}' if !in_quote => {
                flush(source, &mut statements, &mut current, start_offset);
                statements.push(("}".to_owned(), span_for_offset(source, offset)));
            }
            '\n' if !in_quote && current.trim_start().starts_with("@script") => {
                flush(source, &mut statements, &mut current, start_offset);
            }
            _ => current.push(ch),
        }
    }

    flush(source, &mut statements, &mut current, start_offset);
    statements
}

fn flush(
    source: &str,
    statements: &mut Vec<(String, SourceSpan)>,
    current: &mut String,
    start_offset: usize,
) {
    let statement = current.trim();
    if !statement.is_empty() {
        statements.push((statement.to_owned(), span_for_offset(source, start_offset)));
    }
    current.clear();
}

fn parse_statement(statement: &str) -> Vec<AstNode> {
    let statement = statement.trim().trim_end_matches(';').trim();
    if statement.is_empty() {
        return Vec::new();
    }

    if statement == "}" {
        return vec![command_node("endif", Vec::new(), Vec::new())];
    }

    if statement == "else" {
        return vec![command_node("else", Vec::new(), Vec::new())];
    }

    if let Some(command) = statement.strip_prefix('@') {
        return vec![parse_command(command)];
    }

    if let Some(label) = statement.strip_prefix("label ") {
        return vec![AstNode::Label(
            label
                .trim()
                .trim_matches('"')
                .trim_end_matches(':')
                .to_owned(),
        )];
    }

    if let Some((speaker, text)) = statement.split_once(':') {
        if !speaker.contains('(') {
            return vec![
                AstNode::Speaker(speaker.trim().to_owned()),
                AstNode::Text(text.trim().to_owned()),
            ];
        }
    }

    if let Some(node) = parse_call_statement(statement) {
        return node;
    }

    let word = first_word(statement);
    if is_command_name(word) {
        return vec![parse_command(statement)];
    }

    vec![AstNode::Text(statement.to_owned())]
}

fn parse_call_statement(statement: &str) -> Option<Vec<AstNode>> {
    let open = statement.find('(')?;
    let close = statement.rfind(')')?;
    if close < open {
        return None;
    }

    let name = statement[..open].trim();
    let (mut args, mut attributes) = parse_call_parts(&statement[open + 1..close]);

    match name {
        "label" => Some(vec![AstNode::Label(
            take_attr(&mut attributes, "name")
                .or_else(|| take_attr(&mut attributes, "id"))
                .or_else(|| args.first().cloned())
                .unwrap_or_default(),
        )]),
        "say" => {
            let speaker = take_attr(&mut attributes, "speaker").or_else(|| args.first().cloned());
            let text = take_attr(&mut attributes, "text").or_else(|| args.get(1).cloned());
            if let Some(speaker) = speaker {
                let mut nodes = vec![AstNode::Speaker(speaker)];
                if let Some(text) = text {
                    nodes.push(AstNode::Text(text));
                }
                Some(nodes)
            } else {
                text.map(|text| vec![AstNode::Text(text)])
            }
        }
        "choice" => {
            if let Some(text) = take_attr(&mut attributes, "text") {
                args.insert(0, text);
            }
            Some(vec![command_node(name, args, attributes)])
        }
        _ => Some(vec![command_node(name, args, attributes)]),
    }
}

fn parse_call_parts(input: &str) -> (Vec<String>, Vec<Attribute>) {
    let mut args = Vec::new();
    let mut attributes = Vec::new();

    for part in split_comma_parts(input) {
        if let Some((key, value)) = part.split_once('=') {
            attributes.push(Attribute {
                key: key.trim().to_owned(),
                value: unquote(value),
            });
        } else if !part.trim().is_empty() {
            args.push(unquote(&part));
        }
    }

    (args, attributes)
}

fn split_comma_parts(input: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;
    let mut escaped = false;

    for ch in input.chars() {
        if escaped {
            current.push(ch);
            escaped = false;
            continue;
        }

        match ch {
            '\\' if in_quote => {
                current.push(ch);
                escaped = true;
            }
            '"' => {
                in_quote = !in_quote;
                current.push(ch);
            }
            ',' if !in_quote => {
                parts.push(std::mem::take(&mut current));
            }
            _ => current.push(ch),
        }
    }

    if !current.trim().is_empty() {
        parts.push(current);
    }
    parts
}

fn take_attr(attributes: &mut Vec<Attribute>, key: &str) -> Option<String> {
    let index = attributes
        .iter()
        .position(|attribute| attribute.key == key)?;
    Some(attributes.remove(index).value)
}
