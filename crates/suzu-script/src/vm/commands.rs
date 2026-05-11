use serde::{Deserialize, Serialize};
use suzu_core::{Color, Vec2};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Command {
    Bg {
        file: String,
        time_ms: u32,
        method: Transition,
    },
    Char {
        name: String,
        face: String,
        pos: Position,
        size: Vec2,
        flip_x: bool,
        layer: i32,
    },
    HideChar {
        name: String,
    },
    Text {
        speaker: Option<String>,
        content: String,
    },
    Choice {
        options: Vec<ChoiceOption>,
    },
    If {
        condition: String,
        then_commands: Vec<Command>,
        else_commands: Vec<Command>,
    },
    Label {
        name: String,
    },
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
    Wait {
        duration_ms: u32,
    },
    MessageBox {
        visible: bool,
    },
    Jump {
        label: String,
    },
    Call {
        label: String,
    },
    Return,
    SetVar {
        name: String,
        value: String,
    },
    Anim {
        target: String,
        animation: Animation,
    },
    Fx {
        effect: VisualEffect,
    },
    SaveName {
        text: String,
    },
    AutoSave,
    Custom {
        name: String,
        args: Vec<String>,
        attributes: Vec<CustomCommandAttribute>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomCommandAttribute {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Transition {
    Instant,
    CrossFade { duration_ms: u32 },
    FadeThroughColor { color: Color, duration_ms: u32 },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Position {
    Left,
    Center,
    Right,
    Custom(Vec2),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChoiceOption {
    pub text: String,
    pub goto: String,
    pub condition: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Animation {
    pub kind: AnimationKind,
    pub duration_ms: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnimationKind {
    Shake { intensity: f32 },
    MoveTo { position: Vec2 },
    Zoom { center: Vec2, scale: f32 },
    FadeTo { opacity: f32 },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VisualEffect {
    Flash { color: Color, duration_ms: u32 },
    Quake { intensity: f32, duration_ms: u32 },
}
