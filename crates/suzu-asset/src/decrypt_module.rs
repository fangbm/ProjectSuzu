use std::{fs, path::Path};

use anyhow::{bail, Context, Result};
use serde::Deserialize;

use crate::{Xp3Decryptor, Xp3Options};

#[derive(Debug, Clone)]
pub struct DecryptModule {
    name: String,
    xp3_options: Xp3Options,
}

impl DecryptModule {
    pub fn from_json_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let source = fs::read_to_string(path)
            .with_context(|| format!("failed to read decrypt module {}", path.display()))?;
        Self::from_json_str(&source)
            .with_context(|| format!("failed to parse decrypt module {}", path.display()))
    }

    pub fn from_json_str(source: &str) -> Result<Self> {
        let file: DecryptModuleFile =
            serde_json::from_str(source).context("decrypt module JSON is invalid")?;
        if let Some(format) = &file.format {
            if format != "suzu.decrypt-module.v1" {
                bail!("unsupported decrypt module format `{format}`");
            }
        }
        let decryptors = file
            .xp3
            .decryptors
            .into_iter()
            .map(DecryptorSpec::into_decryptor)
            .collect::<Result<Vec<_>>>()?;
        let decryptor = match decryptors.as_slice() {
            [] => Xp3Decryptor::default(),
            [decryptor] => decryptor.clone(),
            _ => Xp3Decryptor::Pipeline(decryptors),
        };
        Ok(Self {
            name: file
                .name
                .unwrap_or_else(|| "unnamed decrypt module".to_owned()),
            xp3_options: Xp3Options { decryptor },
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn xp3_options(&self) -> Xp3Options {
        self.xp3_options.clone()
    }
}

#[derive(Debug, Deserialize)]
struct DecryptModuleFile {
    #[serde(default)]
    format: Option<String>,
    #[serde(default)]
    name: Option<String>,
    xp3: Xp3ModuleConfig,
}

#[derive(Debug, Deserialize)]
struct Xp3ModuleConfig {
    #[serde(default)]
    decryptors: Vec<DecryptorSpec>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum DecryptorSpec {
    Xor { key: String },
    XorAfterInflate { key: String },
    NameXor { key: String },
}

impl DecryptorSpec {
    fn into_decryptor(self) -> Result<Xp3Decryptor> {
        match self {
            Self::Xor { key } => Ok(Xp3Decryptor::Xor {
                key: parse_byte_key(&key)?,
            }),
            Self::XorAfterInflate { key } => Ok(Xp3Decryptor::XorAfterInflate {
                key: parse_byte_key(&key)?,
            }),
            Self::NameXor { key } => Ok(Xp3Decryptor::NameXor {
                key: parse_byte_key(&key)?,
            }),
        }
    }
}

fn parse_byte_key(value: &str) -> Result<u8> {
    let trimmed = value.trim();
    let hex = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"));
    if let Some(hex) = hex {
        return u8::from_str_radix(hex, 16)
            .with_context(|| format!("decryptor key must be a byte, got `{value}`"));
    }
    u8::from_str_radix(trimmed, 16)
        .or_else(|_| trimmed.parse::<u8>())
        .with_context(|| format!("decryptor key must be a byte, got `{value}`"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_xor_module() {
        let module = DecryptModule::from_json_str(
            r#"{
                "format": "suzu.decrypt-module.v1",
                "name": "test xor",
                "xp3": { "decryptors": [{ "type": "xor", "key": "5A" }] }
            }"#,
        )
        .unwrap();
        assert_eq!(module.name(), "test xor");
        assert!(matches!(
            module.xp3_options().decryptor,
            Xp3Decryptor::Xor { key: 0x5a }
        ));
    }

    #[test]
    fn loads_decryptor_pipeline() {
        let module = DecryptModule::from_json_str(
            r#"{
                "xp3": {
                    "decryptors": [
                        { "type": "name_xor", "key": "0x12" },
                        { "type": "xor_after_inflate", "key": "90" }
                    ]
                }
            }"#,
        )
        .unwrap();
        let Xp3Decryptor::Pipeline(decryptors) = module.xp3_options().decryptor else {
            panic!("expected pipeline decryptor");
        };
        assert_eq!(decryptors.len(), 2);
    }
}
