#![cfg_attr(windows, windows_subsystem = "windows")]

mod error_dialog;

use std::{
    env,
    ffi::OsString,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};
use suzu_platform::run_desktop;
use suzu_project::{
    check_project, load_project, ProjectLoadOptions, DEFAULT_ENTRY, LEGACY_ENTRY,
    PROJECT_CONFIG_FILE,
};

fn main() {
    if let Err(error) = run() {
        if std::env::args_os().any(|arg| arg == "--check") {
            eprintln!("Project Suzu Player check FAILED");
            eprintln!("reason: {error:#}");
            std::process::exit(1);
        }
        error_dialog::report_startup_error(&error);
    }
}

fn run() -> Result<()> {
    let args = resolve_default_project_root(parse_args(std::env::args_os().skip(1))?)?;
    if args.check {
        let report = check_project(
            &args.project_root,
            ProjectLoadOptions {
                entry_override: args.entry_override,
            },
        )
        .context("player check failed")?;
        println!("Project Suzu Player check OK");
        println!("version: {}", env!("CARGO_PKG_VERSION"));
        println!("project: {}", report.root.display());
        println!("entry: {}", report.entry_path.display());
        println!("assets: {}", report.registered_assets);
        println!("packages: {}", report.registered_packages);
        return Ok(());
    }

    let loaded = load_project(
        &args.project_root,
        ProjectLoadOptions {
            entry_override: args.entry_override,
        },
    )
    .context("failed to load Project Suzu project")?;
    run_desktop(loaded.app.config.window.clone(), loaded.app)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PlayerArgs {
    project_root: PathBuf,
    project_root_explicit: bool,
    entry_override: Option<PathBuf>,
    check: bool,
}

fn parse_args<I>(args: I) -> Result<PlayerArgs>
where
    I: IntoIterator<Item = OsString>,
{
    let mut project_root = None;
    let mut entry_override = None;
    let mut check = false;
    let mut args = args.into_iter().peekable();

    while let Some(arg) = args.next() {
        if arg == "--check" {
            check = true;
            continue;
        }
        if arg == "--entry" {
            let Some(value) = args.next() else {
                bail!("--entry requires a script path");
            };
            entry_override = Some(PathBuf::from(value));
            continue;
        }
        if arg == "--help" || arg == "-h" {
            print_usage();
            std::process::exit(0);
        }
        if arg.to_string_lossy().starts_with('-') {
            bail!("unknown option `{}`", arg.to_string_lossy());
        }
        if project_root.is_some() {
            bail!("only one project root can be provided");
        }
        project_root = Some(PathBuf::from(arg));
    }

    let project_root_explicit = project_root.is_some();
    Ok(PlayerArgs {
        project_root: project_root.unwrap_or_else(|| PathBuf::from(".")),
        project_root_explicit,
        entry_override,
        check,
    })
}

fn resolve_default_project_root(mut args: PlayerArgs) -> Result<PlayerArgs> {
    if args.project_root_explicit
        || args.entry_override.is_some()
        || looks_like_project_root(&args.project_root)
    {
        return Ok(args);
    }

    if let Some(template_root) = find_bundled_template_project(&args.project_root) {
        args.project_root = template_root;
        return Ok(args);
    }

    bail!(
        "no Project Suzu project was found in `{}`. Pass a project folder, or run from a folder containing `{}` and `{}`. Example: suzu-player templates\\krkr-like-vn",
        args.project_root.display(),
        PROJECT_CONFIG_FILE,
        DEFAULT_ENTRY
    );
}

fn find_bundled_template_project(root: &Path) -> Option<PathBuf> {
    let mut candidates = vec![root.join("templates").join("krkr-like-vn")];
    if let Ok(exe_path) = env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            candidates.push(exe_dir.join("templates").join("krkr-like-vn"));
        }
    }

    candidates
        .into_iter()
        .find(|candidate| looks_like_project_root(candidate))
}

fn looks_like_project_root(root: &Path) -> bool {
    root.join(PROJECT_CONFIG_FILE).is_file()
        || root.join(DEFAULT_ENTRY).is_file()
        || root.join(LEGACY_ENTRY).is_file()
}

fn print_usage() {
    println!("usage: suzu-player [project-root] [--check] [--entry scenario/prologue.szs]");
    println!();
    println!("examples:");
    println!("  suzu-player");
    println!("  suzu-player templates\\krkr-like-vn");
    println!("  suzu-player --check templates\\krkr-like-vn");
    println!("  suzu-player templates\\krkr-like-vn --entry scenario\\chapter1.szs");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_check_before_root() {
        let args = parse_args([OsString::from("--check"), OsString::from("game")]).unwrap();

        assert!(args.check);
        assert!(args.project_root_explicit);
        assert_eq!(args.project_root, PathBuf::from("game"));
    }

    #[test]
    fn parses_check_after_root_and_entry_override() {
        let args = parse_args([
            OsString::from("game"),
            OsString::from("--entry"),
            OsString::from("scenario/prologue.szs"),
            OsString::from("--check"),
        ])
        .unwrap();

        assert!(args.check);
        assert!(args.project_root_explicit);
        assert_eq!(args.project_root, PathBuf::from("game"));
        assert_eq!(
            args.entry_override,
            Some(PathBuf::from("scenario/prologue.szs"))
        );
    }

    #[test]
    fn parses_default_root_as_implicit() {
        let args = parse_args([]).unwrap();

        assert!(!args.project_root_explicit);
        assert_eq!(args.project_root, PathBuf::from("."));
    }

    #[test]
    fn implicit_empty_root_uses_bundled_template_when_present() {
        let root = unique_temp_dir("implicit-template");
        let template = root.join("templates").join("krkr-like-vn");
        std::fs::create_dir_all(template.join("scenario")).unwrap();
        std::fs::write(
            template.join("scenario").join("main.szs"),
            "@script version=1\n",
        )
        .unwrap();

        let args = PlayerArgs {
            project_root: root.clone(),
            project_root_explicit: false,
            entry_override: None,
            check: false,
        };
        let resolved = resolve_default_project_root(args).unwrap();

        assert_eq!(resolved.project_root, template);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn explicit_empty_root_does_not_use_bundled_template() {
        let root = unique_temp_dir("explicit-root");
        let template = root.join("templates").join("krkr-like-vn");
        std::fs::create_dir_all(template.join("scenario")).unwrap();
        std::fs::write(
            template.join("scenario").join("main.szs"),
            "@script version=1\n",
        )
        .unwrap();

        let args = PlayerArgs {
            project_root: root.clone(),
            project_root_explicit: true,
            entry_override: None,
            check: false,
        };
        let resolved = resolve_default_project_root(args).unwrap();

        assert_eq!(resolved.project_root, root);
        std::fs::remove_dir_all(resolved.project_root).unwrap();
    }

    #[test]
    fn implicit_empty_root_reports_clear_error_without_template() {
        let root = unique_temp_dir("missing-template");
        std::fs::create_dir_all(&root).unwrap();
        let args = PlayerArgs {
            project_root: root.clone(),
            project_root_explicit: false,
            entry_override: None,
            check: false,
        };

        let error = resolve_default_project_root(args).unwrap_err().to_string();

        assert!(error.contains("no Project Suzu project was found"));
        assert!(error.contains(PROJECT_CONFIG_FILE));
        std::fs::remove_dir_all(root).unwrap();
    }

    fn unique_temp_dir(name: &str) -> PathBuf {
        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "suzu-player-{name}-{}-{suffix}",
            std::process::id()
        ))
    }
}
