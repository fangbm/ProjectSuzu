use std::{fs, path::Path};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default)]
pub struct FrameStats {
    pub frame_index: u64,
    pub layer_count: usize,
    pub post_process_passes: usize,
    pub shader_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PostProcessSettings {
    pub enabled: bool,
    pub bloom: BloomSettings,
    pub tone_mapping: ToneMappingSettings,
}

impl Default for PostProcessSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            bloom: BloomSettings::default(),
            tone_mapping: ToneMappingSettings::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BloomSettings {
    pub enabled: bool,
    pub threshold: f32,
    pub intensity: f32,
    pub radius: f32,
}

impl Default for BloomSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            threshold: 1.0,
            intensity: 0.35,
            radius: 4.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ToneMappingSettings {
    pub enabled: bool,
    pub exposure: f32,
    pub gamma: f32,
}

impl Default for ToneMappingSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            exposure: 1.0,
            gamma: 2.2,
        }
    }
}

impl PostProcessSettings {
    pub fn active_pass_count(&self) -> usize {
        if !self.enabled {
            return 0;
        }

        usize::from(self.bloom.enabled) + usize::from(self.tone_mapping.enabled)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderSource {
    pub id: String,
    pub source: String,
}

impl ShaderSource {
    pub fn from_wgsl_file(id: impl Into<String>, path: impl AsRef<Path>) -> Result<Self> {
        let id = id.into();
        let path = path.as_ref();
        let source = fs::read_to_string(path)
            .with_context(|| format!("failed to read shader {}", path.display()))?;
        Self::from_wgsl_source(id, source)
    }

    pub fn from_wgsl_source(id: impl Into<String>, source: impl Into<String>) -> Result<Self> {
        let id = id.into();
        let source = source.into();
        validate_wgsl_source(&id, &source)?;
        Ok(Self { id, source })
    }
}

#[derive(Debug, Default)]
pub struct Renderer {
    frame_index: u64,
    post_process: PostProcessSettings,
    shaders: Vec<ShaderSource>,
}

impl Renderer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn begin_frame(&mut self, layer_count: usize) -> FrameStats {
        self.frame_index += 1;
        FrameStats {
            frame_index: self.frame_index,
            layer_count,
            post_process_passes: self.post_process.active_pass_count(),
            shader_count: self.shaders.len(),
        }
    }

    pub fn post_process(&self) -> &PostProcessSettings {
        &self.post_process
    }

    pub fn set_post_process(&mut self, settings: PostProcessSettings) {
        self.post_process = settings;
    }

    pub fn shaders(&self) -> &[ShaderSource] {
        &self.shaders
    }

    pub fn register_shader(&mut self, shader: ShaderSource) {
        if let Some(existing) = self
            .shaders
            .iter_mut()
            .find(|existing| existing.id == shader.id)
        {
            *existing = shader;
        } else {
            self.shaders.push(shader);
        }
    }

    pub fn load_shader_file(
        &mut self,
        id: impl Into<String>,
        path: impl AsRef<Path>,
    ) -> Result<()> {
        let shader = ShaderSource::from_wgsl_file(id, path)?;
        self.register_shader(shader);
        Ok(())
    }
}

fn validate_wgsl_source(id: &str, source: &str) -> Result<()> {
    let trimmed = source.trim();
    if trimmed.is_empty() {
        bail!("shader `{id}` is empty");
    }
    if !trimmed.contains("@fragment")
        && !trimmed.contains("@vertex")
        && !trimmed.contains("@compute")
    {
        bail!("shader `{id}` does not contain a WGSL entry point");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn post_process_settings_report_active_passes() {
        let mut settings = PostProcessSettings::default();
        assert_eq!(settings.active_pass_count(), 1);

        settings.bloom.enabled = true;
        assert_eq!(settings.active_pass_count(), 2);

        settings.enabled = false;
        assert_eq!(settings.active_pass_count(), 0);
    }

    #[test]
    fn renderer_frame_stats_include_post_process_and_shaders() {
        let mut renderer = Renderer::new();
        let mut settings = PostProcessSettings::default();
        settings.bloom.enabled = true;
        renderer.set_post_process(settings);
        renderer.register_shader(
            ShaderSource::from_wgsl_source("main", "@fragment fn fs() {}").unwrap(),
        );

        let stats = renderer.begin_frame(4);

        assert_eq!(stats.frame_index, 1);
        assert_eq!(stats.layer_count, 4);
        assert_eq!(stats.post_process_passes, 2);
        assert_eq!(stats.shader_count, 1);
    }

    #[test]
    fn shader_source_rejects_empty_or_entryless_sources() {
        assert!(ShaderSource::from_wgsl_source("empty", "").is_err());
        assert!(ShaderSource::from_wgsl_source("no_entry", "let x = 1;").is_err());
    }

    #[test]
    fn shader_source_loads_wgsl_file() {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "suzu-shader-{}.wgsl",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::write(&path, "@vertex fn vs() {}").unwrap();

        let shader = ShaderSource::from_wgsl_file("vertex", &path).unwrap();

        assert_eq!(shader.id, "vertex");
        assert!(shader.source.contains("@vertex"));
        let _ = fs::remove_file(path);
    }
}
