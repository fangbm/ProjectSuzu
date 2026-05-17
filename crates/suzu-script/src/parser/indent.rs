use super::{
    document::{
        command_node, first_word, indentation, is_command_name, parse_command, push_classic_line,
        span_for_line,
    },
    AstNode, ScriptDocument, ScriptSyntax, SourceSpan,
};

pub(super) fn parse_indent_script(source: &str, syntax: ScriptSyntax) -> ScriptDocument {
    let mut nodes = Vec::new();
    let mut spans = Vec::new();
    let mut if_stack = Vec::new();

    for (line_index, raw_line) in source.lines().enumerate() {
        let mut line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        let span = span_for_line(line_index, raw_line);
        let indent = indentation(raw_line);
        line = line.trim_end_matches(';').trim();
        let is_else = line.trim_end_matches(':').trim() == "else";

        if is_else {
            close_indent_blocks(&mut nodes, &mut spans, &mut if_stack, indent, span, true);
            push_node(
                &mut nodes,
                &mut spans,
                command_node("else", Vec::new(), Vec::new()),
                span,
            );
            continue;
        }

        close_indent_blocks(&mut nodes, &mut spans, &mut if_stack, indent, span, false);
        let line = line.trim_end_matches(':').trim();

        if line.starts_with('@')
            || line.starts_with(';')
            || line.starts_with('#')
            || line.starts_with('*')
        {
            push_classic_line(&mut nodes, &mut spans, line, span);
            if is_if_line(line.strip_prefix('@').unwrap_or(line)) {
                if_stack.push(indent);
            }
            continue;
        }

        if let Some(label) = line.strip_prefix("label ") {
            push_node(
                &mut nodes,
                &mut spans,
                AstNode::Label(label.trim().trim_end_matches(':').to_owned()),
                span,
            );
            continue;
        }

        let word = first_word(line);
        if is_command_name(word) {
            push_node(&mut nodes, &mut spans, parse_command(line), span);
            if word == "if" {
                if_stack.push(indent);
            }
            continue;
        }

        if let Some((speaker, text)) = line.split_once(':') {
            push_node(
                &mut nodes,
                &mut spans,
                AstNode::Speaker(speaker.trim().to_owned()),
                span,
            );
            let text = text.trim();
            if !text.is_empty() {
                push_node(&mut nodes, &mut spans, AstNode::Text(text.to_owned()), span);
            }
        } else {
            push_node(&mut nodes, &mut spans, AstNode::Text(line.to_owned()), span);
        }
    }

    let eof_span = SourceSpan {
        line: source.lines().count().saturating_add(1),
        column: 1,
    };
    while if_stack.pop().is_some() {
        push_node(
            &mut nodes,
            &mut spans,
            command_node("endif", Vec::new(), Vec::new()),
            eof_span,
        );
    }

    ScriptDocument {
        syntax,
        nodes,
        spans,
    }
}

fn close_indent_blocks(
    nodes: &mut Vec<AstNode>,
    spans: &mut Vec<SourceSpan>,
    if_stack: &mut Vec<usize>,
    indent: usize,
    span: SourceSpan,
    keep_peer_if_for_else: bool,
) {
    while let Some(header_indent) = if_stack.last().copied() {
        let should_close = if keep_peer_if_for_else {
            indent < header_indent
        } else {
            indent <= header_indent
        };
        if !should_close {
            break;
        }
        if_stack.pop();
        push_node(
            nodes,
            spans,
            command_node("endif", Vec::new(), Vec::new()),
            span,
        );
    }
}

fn push_node(
    nodes: &mut Vec<AstNode>,
    spans: &mut Vec<SourceSpan>,
    node: AstNode,
    span: SourceSpan,
) {
    nodes.push(node);
    spans.push(span);
}

fn is_if_line(line: &str) -> bool {
    first_word(line) == "if"
}
