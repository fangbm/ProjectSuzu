use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use suzu_core::Vec2;
use suzu_script::parser::SourceSpan;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u64);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EditorDocument {
    pub version: u32,
    pub source_path: Option<PathBuf>,
    pub metadata: EditorMetadata,
    pub nodes: Vec<EditorNode>,
    pub edges: Vec<EditorEdge>,
    pub comments: Vec<EditorComment>,
}

impl Default for EditorDocument {
    fn default() -> Self {
        Self {
            version: 1,
            source_path: None,
            metadata: EditorMetadata::default(),
            nodes: Vec::new(),
            edges: Vec::new(),
            comments: Vec::new(),
        }
    }
}

impl EditorDocument {
    pub fn next_node_id(&self) -> NodeId {
        NodeId(self.nodes.iter().map(|node| node.id.0).max().unwrap_or(0) + 1)
    }

    pub fn push_node(&mut self, kind: EditorNodeKind, span: Option<SourceSpan>) -> NodeId {
        let id = self.next_node_id();
        self.nodes.push(EditorNode::new(id, kind, span));
        id
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EditorMetadata {
    pub title: String,
    pub script_version: u32,
}

impl Default for EditorMetadata {
    fn default() -> Self {
        Self {
            title: "Untitled".to_owned(),
            script_version: suzu_script::CURRENT_SCRIPT_FORMAT_VERSION,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EditorNode {
    pub id: NodeId,
    pub kind: EditorNodeKind,
    pub title: String,
    pub source_span: Option<SourceSpan>,
    pub layout: NodeLayout,
    pub locked: bool,
}

impl EditorNode {
    pub fn new(id: NodeId, kind: EditorNodeKind, source_span: Option<SourceSpan>) -> Self {
        Self {
            title: kind.default_title(),
            id,
            kind,
            source_span,
            layout: NodeLayout::default(),
            locked: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeLayout {
    pub x: f32,
    pub y: f32,
    pub collapsed: bool,
}

impl Default for NodeLayout {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            collapsed: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EditorNodeKind {
    ScriptHeader {
        version: u32,
    },
    Label {
        name: String,
    },
    Dialogue {
        speaker: Option<String>,
        text: String,
    },
    Background {
        file: String,
        method: TransitionForm,
        time_ms: u32,
    },
    Character {
        name: String,
        face: Option<String>,
        position: PositionForm,
        size: Option<Vec2>,
        layer: i32,
        flip: bool,
    },
    HideCharacter {
        name: String,
    },
    Animation {
        target: String,
        form: AnimationForm,
    },
    Effect {
        form: EffectForm,
    },
    Choice {
        options: Vec<ChoiceOptionForm>,
    },
    SetVariable {
        name: String,
        value: String,
    },
    If {
        condition: String,
        then_nodes: Vec<NodeId>,
        else_nodes: Vec<NodeId>,
    },
    Jump {
        label: String,
    },
    Call {
        label: String,
    },
    Return,
    Wait {
        time_ms: u32,
    },
    Audio {
        form: AudioForm,
    },
    MessageBox {
        visible: bool,
    },
    SaveName {
        text: String,
    },
    AutoSave,
    CustomCommand {
        name: String,
        args: Vec<CommandArgForm>,
    },
    RawText {
        source: String,
    },
}

impl EditorNodeKind {
    pub fn default_title(&self) -> String {
        match self {
            Self::ScriptHeader { .. } => "Script Header".to_owned(),
            Self::Label { name } => format!("Label: {name}"),
            Self::Dialogue { speaker, .. } => speaker.as_ref().map_or_else(
                || "Narration".to_owned(),
                |speaker| format!("Dialogue: {speaker}"),
            ),
            Self::Background { file, .. } => format!("Background: {file}"),
            Self::Character { name, .. } => format!("Character: {name}"),
            Self::HideCharacter { name } => format!("Hide Character: {name}"),
            Self::Animation { target, .. } => format!("Animation: {target}"),
            Self::Effect { .. } => "Effect".to_owned(),
            Self::Choice { .. } => "Choice".to_owned(),
            Self::SetVariable { name, .. } => format!("Set: {name}"),
            Self::If { condition, .. } => format!("If: {condition}"),
            Self::Jump { label } => format!("Jump: {label}"),
            Self::Call { label } => format!("Call: {label}"),
            Self::Return => "Return".to_owned(),
            Self::Wait { .. } => "Wait".to_owned(),
            Self::Audio { .. } => "Audio".to_owned(),
            Self::MessageBox { visible } => if *visible {
                "Show Message Box"
            } else {
                "Hide Message Box"
            }
            .to_owned(),
            Self::SaveName { text } => format!("Save Name: {text}"),
            Self::AutoSave => "Auto Save".to_owned(),
            Self::CustomCommand { name, .. } => format!("Custom: @{name}"),
            Self::RawText { .. } => "Raw Text".to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TransitionForm {
    Instant,
    CrossFade { duration_ms: u32 },
    FadeThroughColor { color: String, duration_ms: u32 },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PositionForm {
    Left,
    Center,
    Right,
    Custom { x: f32, y: f32 },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnimationForm {
    Shake {
        intensity: f32,
        duration_ms: u32,
    },
    MoveTo {
        x: f32,
        y: f32,
        duration_ms: u32,
    },
    Zoom {
        center_x: f32,
        center_y: f32,
        scale: f32,
        duration_ms: u32,
    },
    FadeTo {
        opacity: f32,
        duration_ms: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EffectForm {
    Flash { color: String, duration_ms: u32 },
    Quake { intensity: f32, duration_ms: u32 },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AudioForm {
    PlayBgm {
        file: String,
        looping: bool,
        fadein_ms: u32,
    },
    StopBgm {
        fadeout_ms: u32,
    },
    PlayVoice {
        file: String,
        fadein_ms: u32,
    },
    CueVoice {
        file: String,
        fadein_ms: u32,
    },
    StopVoice {
        fadeout_ms: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChoiceOptionForm {
    pub text: String,
    pub goto: String,
    pub condition: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommandArgForm {
    pub key: Option<String>,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EditorEdge {
    pub from: NodeId,
    pub to: NodeId,
    pub kind: EditorEdgeKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorEdgeKind {
    Sequence,
    ChoiceBranch { option_index: usize },
    ConditionalThen,
    ConditionalElse,
    Jump,
    Call,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorComment {
    pub node: Option<NodeId>,
    pub text: String,
}
