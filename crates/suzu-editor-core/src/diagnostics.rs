use serde::{Deserialize, Serialize};

use crate::document::NodeId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagnosticLevel {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub message: String,
    pub node: Option<NodeId>,
}

impl Diagnostic {
    pub fn error(message: impl Into<String>, node: Option<NodeId>) -> Self {
        Self {
            level: DiagnosticLevel::Error,
            message: message.into(),
            node,
        }
    }

    pub fn warning(message: impl Into<String>, node: Option<NodeId>) -> Self {
        Self {
            level: DiagnosticLevel::Warning,
            message: message.into(),
            node,
        }
    }
}
