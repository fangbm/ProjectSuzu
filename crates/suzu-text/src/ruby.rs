use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RubyAnnotation {
    pub base_range: std::ops::Range<usize>,
    pub ruby: String,
}
