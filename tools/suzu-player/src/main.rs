#![cfg_attr(windows, windows_subsystem = "windows")]

mod error_dialog;

use std::{ffi::OsString, path::PathBuf};

use anyhow::{bail, Context, Result};
use suzu_platform::run_desktop;
use suzu_project::{check_project, load_project, ProjectLoadOptions};

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
    let args = parse_args(std::env::args_os().skip(1))?;
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

    Ok(PlayerArgs {
        project_root: project_root.unwrap_or_else(|| PathBuf::from(".")),
        entry_override,
        check,
    })
}

fn print_usage() {
    println!("usage: suzu-player [project-root] [--check] [--entry scenario/prologue.szs]");
    println!();
    println!("examples:");
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
        assert_eq!(args.project_root, PathBuf::from("game"));
        assert_eq!(
            args.entry_override,
            Some(PathBuf::from("scenario/prologue.szs"))
        );
    }
}
