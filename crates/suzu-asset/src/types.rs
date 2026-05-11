use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetId(pub String);

impl AssetId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl From<&str> for AssetId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for AssetId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetType {
    Texture,
    Audio,
    Script,
    Font,
    Data,
    Unknown,
    Live2DModel,
    Shader,
    Video,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Handle<T> {
    pub id: AssetId,
    _phantom: PhantomData<T>,
}

impl<T> Handle<T> {
    pub fn new(id: AssetId) -> Self {
        Self {
            id,
            _phantom: PhantomData,
        }
    }
}
