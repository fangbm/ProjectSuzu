use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use suzu_app::{GameConfig, SuzuApp, TitleScreenConfig};
use suzu_asset::AssetType;
use suzu_core::Vec2;
use suzu_platform::WindowConfig;

pub const PROJECT_CONFIG_FILE: &str = "game.suzu.toml";
pub const DEFAULT_ENTRY: &str = "scenario/main.szs";
pub const LEGACY_ENTRY: &str = "script/main.szs";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ProjectConfig {
    pub title: String,
    pub subtitle: String,
    pub entry: String,
    pub title_screen: ProjectTitleScreenConfig,
    pub window: ProjectWindowConfig,
    pub assets: ProjectAssetsConfig,
    pub package: ProjectPackageConfig,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            title: "Project Suzu".to_owned(),
            subtitle: "Visual Novel".to_owned(),
            entry: DEFAULT_ENTRY.to_owned(),
            title_screen: ProjectTitleScreenConfig::default(),
            window: ProjectWindowConfig::default(),
            assets: ProjectAssetsConfig::default(),
            package: ProjectPackageConfig::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ProjectTitleScreenConfig {
    pub enabled: bool,
    pub background: Option<String>,
}

impl Default for ProjectTitleScreenConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            background: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ProjectWindowConfig {
    pub title: Option<String>,
    pub width: f32,
    pub height: f32,
    pub resizable: bool,
}

impl Default for ProjectWindowConfig {
    fn default() -> Self {
        Self {
            title: None,
            width: 1280.0,
            height: 720.0,
            resizable: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ProjectAssetsConfig {
    pub roots: Vec<String>,
}

impl Default for ProjectAssetsConfig {
    fn default() -> Self {
        Self {
            roots: vec!["assets".to_owned()],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ProjectPackageConfig {
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ProjectLoadOptions {
    pub entry_override: Option<PathBuf>,
}

pub struct LoadedProject {
    pub root: PathBuf,
    pub config_path: Option<PathBuf>,
    pub config: ProjectConfig,
    pub entry_path: PathBuf,
    pub asset_roots: Vec<PathBuf>,
    pub package_files: Vec<PathBuf>,
    pub registered_assets: usize,
    pub registered_packages: usize,
    pub app: SuzuApp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectCheck {
    pub root: PathBuf,
    pub config_path: Option<PathBuf>,
    pub entry_path: PathBuf,
    pub registered_assets: usize,
    pub registered_packages: usize,
}

impl ProjectCheck {
    pub fn from_loaded(loaded: &LoadedProject) -> Self {
        Self {
            root: loaded.root.clone(),
            config_path: loaded.config_path.clone(),
            entry_path: loaded.entry_path.clone(),
            registered_assets: loaded.registered_assets,
            registered_packages: loaded.registered_packages,
        }
    }

    pub fn warnings(&self) -> Vec<String> {
        project_check_warnings(self)
    }
}

pub fn load_project(root: impl AsRef<Path>, options: ProjectLoadOptions) -> Result<LoadedProject> {
    let root = project_root_from_input(root.as_ref())?;
    let (config_path, mut config) = read_project_config(&root)?;
    apply_legacy_entry_fallback(&root, &mut config);
    let entry_overridden = options.entry_override.is_some();
    if let Some(entry) = options.entry_override {
        config.entry = normalize_config_path(&entry);
    }

    let entry_path = root.join(config.entry.replace('/', std::path::MAIN_SEPARATOR_STR));
    if !entry_path.is_file() {
        if entry_overridden {
            bail!(
                "entry override script does not exist: {} (passed as `{}`)",
                entry_path.display(),
                config.entry
            );
        }
        if config_path.is_none() {
            bail!(
                "project config does not exist: {}; entry script does not exist: {} (expected `{}` or legacy `{}`)",
                root.join(PROJECT_CONFIG_FILE).display(),
                entry_path.display(),
                DEFAULT_ENTRY,
                LEGACY_ENTRY
            );
        }
        bail!(
            "entry script does not exist: {} (configured as `{}`)",
            entry_path.display(),
            config.entry
        );
    }

    let mut app = SuzuApp::new(game_config_from_project(&config));
    let mut registered_assets = 0;
    let mut asset_roots = Vec::new();
    for root_name in &config.assets.roots {
        let asset_root = root.join(root_name.replace('/', std::path::MAIN_SEPARATOR_STR));
        registered_assets += register_asset_root(&mut app, &asset_root)
            .with_context(|| format!("failed to register asset root {}", asset_root.display()))?;
        asset_roots.push(asset_root);
    }

    let mut registered_packages = 0;
    let mut package_files = Vec::new();
    for package_name in &config.package.files {
        let package_path = root.join(package_name.replace('/', std::path::MAIN_SEPARATOR_STR));
        registered_packages += app
            .register_package_file(&package_path)
            .with_context(|| format!("failed to register package {}", package_path.display()))?;
        package_files.push(package_path);
    }

    let source = fs::read_to_string(&entry_path)
        .with_context(|| format!("failed to read entry script {}", entry_path.display()))?;
    app.load_script(&source)
        .with_context(|| format!("failed to compile entry script {}", entry_path.display()))?;
    if !app.config.title_screen.enabled {
        app.advance_until_waiting();
    }

    Ok(LoadedProject {
        root,
        config_path,
        config,
        entry_path,
        asset_roots,
        package_files,
        registered_assets,
        registered_packages,
        app,
    })
}

pub fn check_project(root: impl AsRef<Path>, options: ProjectLoadOptions) -> Result<ProjectCheck> {
    let loaded = load_project(root, options)?;
    Ok(ProjectCheck::from_loaded(&loaded))
}

pub fn write_default_project_config(root: impl AsRef<Path>) -> Result<PathBuf> {
    let root = root.as_ref();
    fs::create_dir_all(root).with_context(|| format!("failed to create {}", root.display()))?;
    let path = root.join(PROJECT_CONFIG_FILE);
    if path.exists() {
        bail!("project config already exists: {}", path.display());
    }
    let config = ProjectConfig::default();
    let toml = toml::to_string_pretty(&config)?;
    fs::write(&path, toml).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(path)
}

pub fn game_config_from_project(config: &ProjectConfig) -> GameConfig {
    GameConfig {
        window: WindowConfig {
            title: config
                .window
                .title
                .clone()
                .unwrap_or_else(|| config.title.clone()),
            logical_size: Vec2::new(config.window.width, config.window.height),
            resizable: config.window.resizable,
        },
        script_entry: config.entry.clone(),
        title_screen: TitleScreenConfig {
            enabled: config.title_screen.enabled,
            title: config.title.clone(),
            subtitle: config.subtitle.clone(),
            background_texture: config.title_screen.background.clone(),
            ..TitleScreenConfig::default()
        },
    }
}

fn project_root_from_input(input: &Path) -> Result<PathBuf> {
    let root = if input
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.eq_ignore_ascii_case(PROJECT_CONFIG_FILE))
    {
        input
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf()
    } else {
        input.to_path_buf()
    };

    if !root.exists() {
        bail!("project root does not exist: {}", root.display());
    }
    if !root.is_dir() {
        bail!("project root is not a directory: {}", root.display());
    }
    Ok(root)
}

fn read_project_config(root: &Path) -> Result<(Option<PathBuf>, ProjectConfig)> {
    let path = root.join(PROJECT_CONFIG_FILE);
    if !path.exists() {
        let config = ProjectConfig {
            entry: detect_default_entry(root),
            ..ProjectConfig::default()
        };
        return Ok((None, config));
    }

    let source =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let config = toml::from_str::<ProjectConfig>(&source)
        .with_context(|| format!("failed to parse {}", path.display()))?;
    Ok((Some(path), config))
}

fn apply_legacy_entry_fallback(root: &Path, config: &mut ProjectConfig) {
    if config.entry == DEFAULT_ENTRY
        && !root.join(DEFAULT_ENTRY).exists()
        && root.join(LEGACY_ENTRY).exists()
    {
        config.entry = LEGACY_ENTRY.to_owned();
    }
}

fn detect_default_entry(root: &Path) -> String {
    if root.join(DEFAULT_ENTRY).is_file() {
        DEFAULT_ENTRY.to_owned()
    } else if root.join(LEGACY_ENTRY).is_file() {
        LEGACY_ENTRY.to_owned()
    } else {
        DEFAULT_ENTRY.to_owned()
    }
}

fn register_asset_root(app: &mut SuzuApp, root: &Path) -> Result<usize> {
    if !root.exists() {
        bail!("asset root does not exist: {}", root.display());
    }
    if !root.is_dir() {
        bail!("asset root is not a directory: {}", root.display());
    }

    let mut count = 0;
    register_asset_dir(app, root, root, &mut count)?;
    Ok(count)
}

fn project_check_warnings(report: &ProjectCheck) -> Vec<String> {
    let mut warnings = Vec::new();
    if report.config_path.is_none() {
        warnings.push(format!(
            "{} not found; using convention defaults. Add this file before publishing a project.",
            PROJECT_CONFIG_FILE
        ));
    }
    if report.registered_assets == 0 && report.registered_packages == 0 {
        warnings.push(
            "no supported assets were found in configured asset roots; add images, audio, fonts, scripts, data, or package files when ready."
                .to_owned(),
        );
    }
    warnings
}

fn register_asset_dir(app: &mut SuzuApp, root: &Path, dir: &Path, count: &mut usize) -> Result<()> {
    for entry in fs::read_dir(dir).with_context(|| format!("failed to read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            register_asset_dir(app, root, &path, count)?;
            continue;
        }

        let Some(kind) = asset_type_from_path(&path) else {
            continue;
        };
        let relative = path
            .strip_prefix(root)
            .with_context(|| format!("failed to relativize {}", path.display()))?;
        let id = asset_id_from_relative(relative);
        app.assets.register_path(id.clone(), kind, path.clone());
        if kind == AssetType::Texture {
            if let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) {
                if stem != id {
                    app.assets
                        .register_path(stem.to_owned(), kind, path.clone());
                }
            }
        }
        *count += 1;
    }
    Ok(())
}

fn asset_type_from_path(path: &Path) -> Option<AssetType> {
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("png" | "jpg" | "jpeg" | "webp") => Some(AssetType::Texture),
        Some("ogg" | "wav" | "mp3" | "flac") => Some(AssetType::Audio),
        Some("szs" | "lua") => Some(AssetType::Script),
        Some("ttf" | "otf" | "ttc") => Some(AssetType::Font),
        Some("mp4" | "webm" | "mkv") => Some(AssetType::Video),
        Some("json" | "ron" | "toml" | "yaml" | "yml") => Some(AssetType::Data),
        _ => None,
    }
}

fn asset_id_from_relative(path: &Path) -> String {
    normalize_config_path(&path.with_extension(""))
}

fn normalize_config_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convention_project_loads_scenario_entry_without_config() {
        let root = test_dir("suzu-project-convention");
        fs::create_dir_all(root.join("scenario")).unwrap();
        fs::create_dir_all(root.join("assets")).unwrap();
        fs::write(root.join("scenario/main.szs"), "@script version=1\n# N\nHi").unwrap();

        let loaded = load_project(&root, ProjectLoadOptions::default()).unwrap();

        assert_eq!(loaded.config_path, None);
        assert_eq!(loaded.config.entry, DEFAULT_ENTRY);
        assert_eq!(loaded.entry_path, root.join("scenario/main.szs"));
        assert!(loaded.app.title_screen_visible());

        let report = check_project(&root, ProjectLoadOptions::default()).unwrap();
        assert!(report
            .warnings()
            .iter()
            .any(|warning| warning.contains(PROJECT_CONFIG_FILE)));
        assert!(report
            .warnings()
            .iter()
            .any(|warning| warning.contains("no supported assets")));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn config_overrides_entry_window_and_title() {
        let root = test_dir("suzu-project-config");
        fs::create_dir_all(root.join("story")).unwrap();
        fs::create_dir_all(root.join("assets")).unwrap();
        fs::write(
            root.join("story/prologue.szs"),
            "@script version=1\n# N\nHi",
        )
        .unwrap();
        fs::write(
            root.join(PROJECT_CONFIG_FILE),
            r#"
title = "Custom Title"
subtitle = "Custom Subtitle"
entry = "story/prologue.szs"

[window]
title = "Window Title"
width = 960
height = 540
resizable = false

[title_screen]
enabled = false
background = "bg/title"
"#,
        )
        .unwrap();

        let loaded = load_project(&root, ProjectLoadOptions::default()).unwrap();

        assert_eq!(loaded.config.title, "Custom Title");
        assert_eq!(loaded.app.config.window.title, "Window Title");
        assert_eq!(
            loaded.app.config.window.logical_size,
            Vec2::new(960.0, 540.0)
        );
        assert!(!loaded.app.config.window.resizable);
        assert!(!loaded.app.config.title_screen.enabled);
        assert!(!loaded.app.title_screen_visible());

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn registers_path_ids_and_texture_stem_aliases() {
        let root = test_dir("suzu-project-assets");
        fs::create_dir_all(root.join("scenario")).unwrap();
        fs::create_dir_all(root.join("assets/bg")).unwrap();
        fs::write(root.join("scenario/main.szs"), "@script version=1\n# N\nHi").unwrap();
        fs::write(root.join("assets/bg/school.png"), [1_u8, 2, 3]).unwrap();

        let loaded = load_project(&root, ProjectLoadOptions::default()).unwrap();

        assert_eq!(loaded.registered_assets, 1);
        assert!(loaded.app.assets.get(&"bg/school".into()).is_some());
        assert!(loaded.app.assets.get(&"school".into()).is_some());

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn registers_assets_and_package_files_together() {
        let root = test_dir("suzu-project-package");
        fs::create_dir_all(root.join("scenario")).unwrap();
        fs::create_dir_all(root.join("assets/bg")).unwrap();
        fs::write(root.join("scenario/main.szs"), "@script version=1\n# N\nHi").unwrap();
        fs::write(root.join("assets/bg/school.png"), [1_u8, 2, 3]).unwrap();
        fs::write(root.join("assets/readme.txt"), b"note").unwrap();
        let manifest = suzu_packer::build_manifest(&root.join("assets")).unwrap();
        suzu_packer::write_archive(&root.join("assets"), manifest, &root.join("data.suzupack"))
            .unwrap();
        fs::write(
            root.join(PROJECT_CONFIG_FILE),
            r#"
entry = "scenario/main.szs"

[assets]
roots = ["assets"]

[package]
files = ["data.suzupack"]
"#,
        )
        .unwrap();

        let loaded = load_project(&root, ProjectLoadOptions::default()).unwrap();

        assert_eq!(loaded.registered_assets, 1);
        assert!(loaded.registered_packages >= 1);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn missing_entry_script_reports_path() {
        let root = test_dir("suzu-project-missing-entry");
        fs::create_dir_all(root.join("assets")).unwrap();

        let error = load_project_error(&root);

        assert!(error.contains("project config does not exist"));
        assert!(error.contains("entry script does not exist"));
        assert!(error.contains(DEFAULT_ENTRY));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn missing_entry_override_reports_override_path() {
        let root = test_dir("suzu-project-missing-entry-override");
        fs::create_dir_all(root.join("scenario")).unwrap();
        fs::create_dir_all(root.join("assets")).unwrap();
        fs::write(root.join("scenario/main.szs"), "@script version=1\n# N\nHi").unwrap();

        let error = match load_project(
            &root,
            ProjectLoadOptions {
                entry_override: Some(PathBuf::from("scenario/missing.szs")),
            },
        ) {
            Ok(_) => panic!("expected load_project to fail"),
            Err(error) => format!("{error:#}"),
        };

        assert!(error.contains("entry override script does not exist"));
        assert!(error.contains("scenario/missing.szs"));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn compile_error_reports_entry_context() {
        let root = test_dir("suzu-project-script-compile-error");
        fs::create_dir_all(root.join("scenario")).unwrap();
        fs::create_dir_all(root.join("assets")).unwrap();
        fs::write(
            root.join("scenario/main.szs"),
            "@script version=999\n# N\nHi",
        )
        .unwrap();

        let error = load_project_error(&root);

        assert!(error.contains("failed to compile entry script"));
        assert!(error.contains("scenario"));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn invalid_toml_reports_config_path() {
        let root = test_dir("suzu-project-invalid-config");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join(PROJECT_CONFIG_FILE), "title = [").unwrap();

        let error = load_project_error(&root);

        assert!(error.contains("failed to parse"));
        assert!(error.contains(PROJECT_CONFIG_FILE));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn missing_configured_asset_root_reports_clear_error() {
        let root = test_dir("suzu-project-missing-assets");
        fs::create_dir_all(root.join("scenario")).unwrap();
        fs::write(root.join("scenario/main.szs"), "@script version=1\n# N\nHi").unwrap();
        fs::write(
            root.join(PROJECT_CONFIG_FILE),
            r#"
[assets]
roots = ["missing-assets"]
"#,
        )
        .unwrap();

        let error = load_project_error(&root);

        assert!(error.contains("asset root does not exist"));
        assert!(error.contains("missing-assets"));

        let _ = fs::remove_dir_all(root);
    }

    fn test_dir(name: &str) -> PathBuf {
        let mut root = std::env::temp_dir();
        root.push(format!(
            "{name}-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::remove_dir_all(&root);
        root
    }

    fn load_project_error(root: &Path) -> String {
        match load_project(root, ProjectLoadOptions::default()) {
            Ok(_) => panic!("expected load_project to fail"),
            Err(error) => format!("{error:#}"),
        }
    }
}
