use std::collections::{HashMap, HashSet};

use crate::{
    diagnostics::Diagnostic,
    document::{EditorDocument, EditorEdge, EditorEdgeKind, EditorNodeKind, NodeId},
};

pub fn rebuild_edges(document: &mut EditorDocument) {
    document.edges.clear();
    let labels = label_index(document);

    for window in document.nodes.windows(2) {
        let [from, to] = window else { continue };
        document.edges.push(EditorEdge {
            from: from.id,
            to: to.id,
            kind: EditorEdgeKind::Sequence,
        });
    }

    for node in &document.nodes {
        match &node.kind {
            EditorNodeKind::Choice { options } => {
                for (index, option) in options.iter().enumerate() {
                    if let Some(target) = labels.get(&option.goto) {
                        document.edges.push(EditorEdge {
                            from: node.id,
                            to: *target,
                            kind: EditorEdgeKind::ChoiceBranch {
                                option_index: index,
                            },
                        });
                    }
                }
            }
            EditorNodeKind::Jump { label } => {
                if let Some(target) = labels.get(label) {
                    document.edges.push(EditorEdge {
                        from: node.id,
                        to: *target,
                        kind: EditorEdgeKind::Jump,
                    });
                }
            }
            EditorNodeKind::Call { label } => {
                if let Some(target) = labels.get(label) {
                    document.edges.push(EditorEdge {
                        from: node.id,
                        to: *target,
                        kind: EditorEdgeKind::Call,
                    });
                }
            }
            _ => {}
        }
    }
}

pub fn analyze_graph(document: &EditorDocument) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let labels = label_index(document);
    let mut seen_labels = HashSet::new();

    for node in &document.nodes {
        match &node.kind {
            EditorNodeKind::Label { name } if !seen_labels.insert(name.clone()) => {
                diagnostics.push(Diagnostic::error(
                    format!("duplicate label `{name}`"),
                    Some(node.id),
                ));
            }
            EditorNodeKind::Choice { options } => {
                if options.is_empty() {
                    diagnostics.push(Diagnostic::warning("choice has no options", Some(node.id)));
                }
                for option in options {
                    if option.goto.is_empty() {
                        diagnostics.push(Diagnostic::error(
                            format!("choice `{}` has no target label", option.text),
                            Some(node.id),
                        ));
                    } else if !labels.contains_key(&option.goto) {
                        diagnostics.push(Diagnostic::error(
                            format!("choice target label `{}` does not exist", option.goto),
                            Some(node.id),
                        ));
                    }
                }
            }
            EditorNodeKind::Jump { label } | EditorNodeKind::Call { label }
                if !labels.contains_key(label) =>
            {
                diagnostics.push(Diagnostic::error(
                    format!("target label `{label}` does not exist"),
                    Some(node.id),
                ));
            }
            EditorNodeKind::If { condition, .. } if condition.trim().is_empty() => {
                diagnostics.push(Diagnostic::error("if condition is empty", Some(node.id)));
            }
            _ => {}
        }
    }

    diagnostics.extend(unreachable_node_diagnostics(document));
    diagnostics
}

fn label_index(document: &EditorDocument) -> HashMap<String, NodeId> {
    document
        .nodes
        .iter()
        .filter_map(|node| match &node.kind {
            EditorNodeKind::Label { name } => Some((name.clone(), node.id)),
            _ => None,
        })
        .collect()
}

fn unreachable_node_diagnostics(document: &EditorDocument) -> Vec<Diagnostic> {
    let Some(first) = document.nodes.first() else {
        return vec![Diagnostic::warning("script has no playable nodes", None)];
    };
    let mut stack = vec![first.id];
    let mut visited = HashSet::new();

    while let Some(id) = stack.pop() {
        if !visited.insert(id) {
            continue;
        }
        stack.extend(
            document
                .edges
                .iter()
                .filter(|edge| edge.from == id)
                .map(|edge| edge.to),
        );
    }

    document
        .nodes
        .iter()
        .filter(|node| !visited.contains(&node.id))
        .map(|node| Diagnostic::warning("node is unreachable", Some(node.id)))
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::import_szs;

    use super::*;

    #[test]
    fn analyzes_missing_choice_label() {
        let doc = import_szs("@choice \"A\" goto=missing", None);
        let diagnostics = analyze_graph(&doc);

        assert!(diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("does not exist")));
    }

    #[test]
    fn creates_choice_edges_to_labels() {
        let doc = import_szs("@choice \"A\" goto=a\n*a\nA", None);

        assert!(doc
            .edges
            .iter()
            .any(|edge| matches!(edge.kind, EditorEdgeKind::ChoiceBranch { .. })));
    }
}
