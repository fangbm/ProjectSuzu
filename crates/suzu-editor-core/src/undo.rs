use crate::document::{EditorDocument, EditorNode, EditorNodeKind, NodeId};

#[derive(Debug, Clone, PartialEq)]
pub enum EditorCommand {
    AddNode {
        node: EditorNode,
        index: usize,
    },
    RemoveNode {
        node: EditorNode,
        index: usize,
    },
    UpdateNode {
        node_id: NodeId,
        before: EditorNodeKind,
        after: EditorNodeKind,
    },
    MoveNode {
        node_id: NodeId,
        from: usize,
        to: usize,
    },
}

impl EditorCommand {
    pub fn apply(&self, document: &mut EditorDocument) {
        match self {
            Self::AddNode { node, index } => {
                let index = (*index).min(document.nodes.len());
                document.nodes.insert(index, node.clone());
            }
            Self::RemoveNode { node, .. } => {
                document.nodes.retain(|existing| existing.id != node.id);
            }
            Self::UpdateNode { node_id, after, .. } => {
                if let Some(node) = document.nodes.iter_mut().find(|node| node.id == *node_id) {
                    node.kind = after.clone();
                    node.title = node.kind.default_title();
                }
            }
            Self::MoveNode { node_id, to, .. } => move_node(document, *node_id, *to),
        }
    }

    pub fn undo(&self, document: &mut EditorDocument) {
        match self {
            Self::AddNode { node, .. } => {
                document.nodes.retain(|existing| existing.id != node.id);
            }
            Self::RemoveNode { node, index } => {
                let index = (*index).min(document.nodes.len());
                document.nodes.insert(index, node.clone());
            }
            Self::UpdateNode {
                node_id, before, ..
            } => {
                if let Some(node) = document.nodes.iter_mut().find(|node| node.id == *node_id) {
                    node.kind = before.clone();
                    node.title = node.kind.default_title();
                }
            }
            Self::MoveNode { node_id, from, .. } => move_node(document, *node_id, *from),
        }
    }
}

fn move_node(document: &mut EditorDocument, node_id: NodeId, to: usize) {
    let Some(from) = document.nodes.iter().position(|node| node.id == node_id) else {
        return;
    };
    let node = document.nodes.remove(from);
    let to = to.min(document.nodes.len());
    document.nodes.insert(to, node);
}
