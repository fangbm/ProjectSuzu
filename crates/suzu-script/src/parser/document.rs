use super::{AstNode, Attribute, ScriptDocument, SourceSpan};

pub fn parse_script(source: &str) -> ScriptDocument {
    let mut nodes = Vec::new();
    let mut spans = Vec::new();

    for (line_index, raw_line) in source.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        spans.push(SourceSpan {
            line: line_index + 1,
            column: raw_line
                .char_indices()
                .find(|(_, ch)| !ch.is_whitespace())
                .map_or(1, |(index, _)| index + 1),
        });

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

    ScriptDocument { nodes, spans }
}

fn parse_command(command: &str) -> AstNode {
    let tokens = tokenize_command(command);
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

fn tokenize_command(command: &str) -> Vec<String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_command_and_text() {
        let doc = parse_script("@bg file=\"school\"\n# 艾琳\n你好");
        assert_eq!(doc.nodes.len(), 3);
        assert!(matches!(&doc.nodes[0], AstNode::Command { name, .. } if name == "bg"));
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
