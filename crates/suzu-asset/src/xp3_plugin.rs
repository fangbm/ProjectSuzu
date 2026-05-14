use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::Arc,
};

use anyhow::{bail, Context, Result};
use serde::Deserialize;

use crate::{Xp3Entry, Xp3Options, Xp3Plugin, Xp3PluginScheme, Xp3Segment};

#[derive(Debug, Clone)]
pub struct Xp3PluginModule {
    name: String,
    options: Xp3Options,
}

impl Xp3PluginModule {
    pub fn from_json_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let source = fs::read_to_string(path)
            .with_context(|| format!("failed to read XP3 plugin module {}", path.display()))?;
        Self::from_json_str_with_base(&source, path.parent())
            .with_context(|| format!("failed to parse XP3 plugin module {}", path.display()))
    }

    pub fn from_json_str(source: &str) -> Result<Self> {
        Self::from_json_str_with_base(source, None)
    }

    fn from_json_str_with_base(source: &str, base_dir: Option<&Path>) -> Result<Self> {
        let file: Xp3PluginModuleFile =
            serde_json::from_str(source).context("XP3 plugin module JSON is invalid")?;
        if let Some(format) = &file.format {
            if format != "suzu.xp3-plugin.v1" {
                bail!("unsupported XP3 plugin module format `{format}`");
            }
        }

        let processors = file
            .xp3
            .processors
            .into_iter()
            .map(|spec| spec.into_plugin(base_dir))
            .collect::<Result<Vec<_>>>()?;
        let plugin = match processors.as_slice() {
            [] => Xp3Plugin::default(),
            [plugin] => plugin.clone(),
            _ => Xp3Plugin::Pipeline(processors),
        };

        Ok(Self {
            name: file
                .name
                .unwrap_or_else(|| "unnamed XP3 plugin module".to_owned()),
            options: Xp3Options { plugin },
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn xp3_options(&self) -> Xp3Options {
        self.options.clone()
    }
}

#[derive(Debug, Deserialize)]
struct Xp3PluginModuleFile {
    #[serde(default)]
    format: Option<String>,
    #[serde(default)]
    name: Option<String>,
    xp3: Xp3ModuleConfig,
}

#[derive(Debug, Deserialize)]
struct Xp3ModuleConfig {
    #[serde(default)]
    processors: Vec<ProcessorSpec>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ProcessorSpec {
    ExternalProcess {
        command: String,
        #[serde(default)]
        args: Vec<String>,
        #[serde(default)]
        stage: ProcessorStage,
    },
}

impl ProcessorSpec {
    fn into_plugin(self, base_dir: Option<&Path>) -> Result<Xp3Plugin> {
        match self {
            Self::ExternalProcess {
                command,
                args,
                stage,
            } => {
                let command = resolve_module_path(&command, base_dir);
                Ok(Xp3Plugin::Custom {
                    scheme: Arc::new(ExternalProcessPlugin {
                        command,
                        args,
                        stage,
                    }),
                })
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ProcessorStage {
    Segment,
    #[default]
    AfterInflate,
}

#[derive(Debug)]
struct ExternalProcessPlugin {
    command: PathBuf,
    args: Vec<String>,
    stage: ProcessorStage,
}

impl Xp3PluginScheme for ExternalProcessPlugin {
    fn process_segment_bytes(
        &self,
        bytes: &mut [u8],
        entry: &Xp3Entry,
        segment: &Xp3Segment,
    ) -> Result<()> {
        if matches!(self.stage, ProcessorStage::Segment) {
            run_external_processor(&self.command, &self.args, bytes, entry, segment)?;
        }
        Ok(())
    }

    fn process_after_inflate(
        &self,
        bytes: &mut [u8],
        entry: &Xp3Entry,
        segment: &Xp3Segment,
    ) -> Result<()> {
        if matches!(self.stage, ProcessorStage::AfterInflate) {
            run_external_processor(&self.command, &self.args, bytes, entry, segment)?;
        }
        Ok(())
    }
}

fn run_external_processor(
    command: &Path,
    args: &[String],
    bytes: &mut [u8],
    entry: &Xp3Entry,
    segment: &Xp3Segment,
) -> Result<()> {
    let mut child = Command::new(command)
        .args(
            args.iter()
                .map(|arg| expand_arg(arg, entry, segment))
                .collect::<Vec<_>>(),
        )
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("failed to start XP3 plugin {}", command.display()))?;
    child
        .stdin
        .as_mut()
        .context("XP3 plugin stdin is not available")?
        .write_all(bytes)
        .context("failed to write bytes to XP3 plugin")?;
    let output = child
        .wait_with_output()
        .context("XP3 plugin did not finish")?;
    if !output.status.success() {
        bail!(
            "XP3 plugin {} failed with status {}: {}",
            command.display(),
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
    }
    if output.stdout.len() != bytes.len() {
        bail!(
            "XP3 plugin {} returned {} bytes for {} byte input",
            command.display(),
            output.stdout.len(),
            bytes.len()
        );
    }
    bytes.copy_from_slice(&output.stdout);
    Ok(())
}

fn resolve_module_path(value: &str, base_dir: Option<&Path>) -> PathBuf {
    let path = PathBuf::from(value);
    if path.is_absolute() {
        return path;
    }
    base_dir.map(|base| base.join(&path)).unwrap_or(path)
}

fn expand_arg(template: &str, entry: &Xp3Entry, segment: &Xp3Segment) -> String {
    template
        .replace("{entry}", &entry.name)
        .replace(
            "{checksum}",
            &entry
                .checksum
                .map(|checksum| checksum.to_string())
                .unwrap_or_default(),
        )
        .replace(
            "{checksum_hex}",
            &entry
                .checksum
                .map(|checksum| format!("{checksum:08x}"))
                .unwrap_or_default(),
        )
        .replace("{original_size}", &entry.original_size.to_string())
        .replace("{packed_size}", &entry.packed_size.to_string())
        .replace("{segment_offset}", &segment.offset.to_string())
        .replace(
            "{segment_original_size}",
            &segment.original_size.to_string(),
        )
        .replace("{segment_packed_size}", &segment.packed_size.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_external_process_module() {
        let module = Xp3PluginModule::from_json_str(
            r#"{
                "format": "suzu.xp3-plugin.v1",
                "name": "test plugin",
                "xp3": {
                    "processors": [
                        {
                            "type": "external_process",
                            "command": "xp3-plugin.exe",
                            "args": ["--entry", "{entry}"],
                            "stage": "after_inflate"
                        }
                    ]
                }
            }"#,
        )
        .unwrap();
        assert_eq!(module.name(), "test plugin");
        assert!(matches!(
            module.xp3_options().plugin,
            Xp3Plugin::Custom { .. }
        ));
    }

    #[test]
    fn expands_external_processor_arguments() {
        let entry = Xp3Entry {
            name: "main/default.tjs".to_owned(),
            protected: true,
            original_size: 12,
            packed_size: 8,
            checksum: Some(0x1234_abcd),
            segments: Vec::new(),
        };
        let segment = Xp3Segment {
            compressed: true,
            offset: 42,
            original_size: 12,
            packed_size: 8,
        };

        assert_eq!(
            expand_arg("{entry}:{checksum_hex}:{segment_offset}", &entry, &segment),
            "main/default.tjs:1234abcd:42"
        );
    }
}
