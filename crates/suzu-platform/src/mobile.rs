use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MobileOs {
    Android,
    Ios,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MobileBuildTarget {
    pub os: MobileOs,
    pub rust_target: String,
    pub package_format: String,
}

impl MobileBuildTarget {
    pub fn android_arm64() -> Self {
        Self {
            os: MobileOs::Android,
            rust_target: "aarch64-linux-android".to_owned(),
            package_format: "apk".to_owned(),
        }
    }

    pub fn ios_arm64() -> Self {
        Self {
            os: MobileOs::Ios,
            rust_target: "aarch64-apple-ios".to_owned(),
            package_format: "app".to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mobile_targets_name_rust_triples() {
        assert_eq!(
            MobileBuildTarget::android_arm64().rust_target,
            "aarch64-linux-android"
        );
        assert_eq!(
            MobileBuildTarget::ios_arm64().rust_target,
            "aarch64-apple-ios"
        );
    }
}
