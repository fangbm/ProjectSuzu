use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioSource {
    File { path: String, looping: bool },
    Memory { bytes: Vec<u8>, looping: bool },
}
