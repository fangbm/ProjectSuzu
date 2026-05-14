use std::{ffi::OsString, path::PathBuf};

use anyhow::{bail, Context};
use suzu_asset::{probe_krkr_directory, Xp3Options, Xp3PluginModule};

use crate::conversion::convert_krkr_package_to_suzu_project;

pub const XP3_PLUGIN_AUTHORIZATION_FLAG: &str = "--i-have-rights-to-process-these-assets";
pub const XP3_PLUGIN_AUTHORIZATION_MESSAGE: &str =
    "Only use XP3 plugins for resources you own or are authorized to process. Do not use plugins to bypass DRM, license checks, or access controls. See LEGAL.md.";

#[derive(Debug)]
pub enum CliAction {
    Handled,
    LaunchGui { initial: PathBuf },
}

pub fn dispatch(args: &[OsString]) -> anyhow::Result<CliAction> {
    if args
        .first()
        .and_then(|arg| arg.to_str())
        .is_some_and(|arg| arg == "--krkr2suzu")
    {
        run_krkr2suzu_cli(&args[1..]).context("krkr2suzu failed")?;
        return Ok(CliAction::Handled);
    }
    if args
        .first()
        .and_then(|arg| arg.to_str())
        .is_some_and(|arg| arg == "--krkr-probe")
    {
        run_krkr_probe_cli(&args[1..]).context("krkr-probe failed")?;
        return Ok(CliAction::Handled);
    }

    Ok(CliAction::LaunchGui {
        initial: args.first().map(PathBuf::from).unwrap_or_default(),
    })
}

fn run_krkr2suzu_cli(args: &[OsString]) -> anyhow::Result<()> {
    if args.len() < 2 {
        bail!(
            "usage: suzu-launcher --krkr2suzu <krkr-folder> <output-folder> [--xp3-plugin <module.json> {XP3_PLUGIN_AUTHORIZATION_FLAG}]"
        );
    }
    let root = PathBuf::from(&args[0]);
    let output = PathBuf::from(&args[1]);
    let mut plugin_path = None;
    let mut plugin_authorized = false;
    let mut index = 2;
    while index < args.len() {
        match args[index].to_string_lossy().as_ref() {
            "--xp3-plugin" if index + 1 < args.len() => {
                plugin_path = Some(PathBuf::from(&args[index + 1]));
                index += 2;
            }
            "--xp3-plugin" => bail!("--xp3-plugin requires a module JSON path"),
            XP3_PLUGIN_AUTHORIZATION_FLAG => {
                plugin_authorized = true;
                index += 1;
            }
            other => bail!("unknown krkr2suzu option `{other}`"),
        }
    }

    let options = if let Some(plugin_path) = plugin_path {
        if !plugin_authorized {
            bail!("{XP3_PLUGIN_AUTHORIZATION_MESSAGE} Pass `{XP3_PLUGIN_AUTHORIZATION_FLAG}` to continue.");
        }
        let module = Xp3PluginModule::from_json_file(plugin_path)?;
        vec![module.xp3_options()]
    } else {
        vec![Xp3Options::default()]
    };

    let summary = convert_krkr_package_to_suzu_project(&root, &output, &options)?;
    println!(
        "Converted {} scripts ({} unreadable) to {} from {} lines, {} commands, {} choices.",
        summary.scripts,
        summary.unreadable,
        summary.script_path.display(),
        summary.lines,
        summary.commands,
        summary.choices
    );
    Ok(())
}

fn run_krkr_probe_cli(args: &[OsString]) -> anyhow::Result<()> {
    let Some(root) = args.first() else {
        bail!("usage: suzu-launcher --krkr-probe <krkr-folder>");
    };
    let report = probe_krkr_directory(PathBuf::from(root))?;
    println!(
        "KRKR archives: {} archives, {} script-like entries, {} protected script-like entries",
        report.archives.len(),
        report.script_entries(),
        report.protected_script_entries()
    );
    if report.has_protected_entries() {
        println!(
            "Compatibility: direct Suzu playback requires an external XP3 plugin for this package."
        );
    }
    for archive in &report.archives {
        let name = archive
            .path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("<xp3>");
        if let Some(error) = &archive.parse_error {
            println!("  {name}: parse error: {error}");
            continue;
        }
        println!(
            "  {name}: {} entries, {} script-like, {} protected",
            archive.entries, archive.script_entries, archive.protected_script_entries
        );
        if !archive.entrypoint_candidates.is_empty() {
            println!(
                "    entrypoint candidates: {}",
                archive.entrypoint_candidates.join(", ")
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_plugin_without_authorization_flag() {
        let args = vec![
            OsString::from("--krkr2suzu"),
            OsString::from("D:\\game"),
            OsString::from("D:\\out"),
            OsString::from("--xp3-plugin"),
            OsString::from("plugin.json"),
        ];

        let error = format!("{:#}", dispatch(&args).unwrap_err());
        assert!(error.contains("krkr2suzu failed"));
        assert!(error.contains(XP3_PLUGIN_AUTHORIZATION_FLAG));
    }

    #[test]
    fn parses_plain_conversion_without_authorization_flag() {
        let args = vec![OsString::from("D:\\game")];

        assert!(matches!(
            dispatch(&args).unwrap(),
            CliAction::LaunchGui { .. }
        ));
    }

    #[test]
    fn accepts_plugin_authorization_flag_before_plugin_loading() {
        let args = vec![
            OsString::from("--krkr2suzu"),
            OsString::from("D:\\game"),
            OsString::from("D:\\out"),
            OsString::from("--xp3-plugin"),
            OsString::from("missing-plugin.json"),
            OsString::from(XP3_PLUGIN_AUTHORIZATION_FLAG),
        ];

        let error = format!("{:#}", dispatch(&args).unwrap_err());
        assert!(error.contains("missing-plugin.json"));
        assert!(!error.contains(XP3_PLUGIN_AUTHORIZATION_FLAG));
    }
}
