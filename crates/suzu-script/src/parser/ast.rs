use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ScriptDocument {
    pub nodes: Vec<AstNode>,
    #[serde(default)]
    pub spans: Vec<SourceSpan>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSpan {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AstNode {
    Command {
        name: String,
        args: Vec<String>,
        attributes: Vec<Attribute>,
    },
    Speaker(String),
    Text(String),
    Label(String),
    Comment(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Attribute {
    pub key: String,
    pub value: String,
}
