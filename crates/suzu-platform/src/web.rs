use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebBuildTarget {
    pub rust_target: String,
    pub output_dir: String,
    pub bootstrap_script: String,
}

impl Default for WebBuildTarget {
    fn default() -> Self {
        Self {
            rust_target: "wasm32-unknown-unknown".to_owned(),
            output_dir: "dist/web".to_owned(),
            bootstrap_script: "bootstrap.js".to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn web_target_defaults_to_wasm() {
        assert_eq!(
            WebBuildTarget::default().rust_target,
            "wasm32-unknown-unknown"
        );
    }
}
