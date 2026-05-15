use std::{ffi::OsString, path::PathBuf};

use anyhow::{bail, Context};
use suzu_asset::{Xp3Archive, Xp3Options, Xp3PluginModule};

pub const XP3_PLUGIN_AUTHORIZATION_FLAG: &str = "--i-have-rights-to-process-these-assets";
pub const XP3_PLUGIN_AUTHORIZATION_MESSAGE: &str =
    "Only use XP3 plugins for resources you own or are authorized to process. Do not use plugins to bypass DRM, license checks, or access controls. See LEGAL.md.";

#[derive(Debug)]
pub enum CliAction {
    Handled,
    LaunchGui { initial_path: String },
}

pub fn dispatch(args: &[OsString]) -> anyhow::Result<CliAction> {
    if args
        .first()
        .and_then(|arg| arg.to_str())
        .is_some_and(|arg| arg == "--check")
    {
        run_check_cli(&args[1..]).context("xp3 viewer check failed")?;
        return Ok(CliAction::Handled);
    }

    Ok(CliAction::LaunchGui {
        initial_path: args
            .first()
            .map(PathBuf::from)
            .unwrap_or_default()
            .display()
            .to_string(),
    })
}

fn run_check_cli(args: &[OsString]) -> anyhow::Result<()> {
    let mut xp3_path = None;
    let mut plugin_path = None;
    let mut plugin_authorized = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].to_string_lossy().as_ref() {
            "--xp3" if index + 1 < args.len() => {
                xp3_path = Some(
                    xp3_path_from_input(&args[index + 1].to_string_lossy())
                        .map_err(|error| anyhow::anyhow!(error))?,
                );
                index += 2;
            }
            "--xp3" => bail!("--xp3 requires an .xp3 path"),
            "--xp3-plugin" if index + 1 < args.len() => {
                plugin_path = Some(PathBuf::from(clean_path_input(
                    &args[index + 1].to_string_lossy(),
                )));
                index += 2;
            }
            "--xp3-plugin" => bail!("--xp3-plugin requires a module JSON path"),
            XP3_PLUGIN_AUTHORIZATION_FLAG => {
                plugin_authorized = true;
                index += 1;
            }
            other => bail!("unknown check option `{other}`"),
        }
    }

    let options = if let Some(plugin_path) = plugin_path {
        if !plugin_authorized {
            bail!(
                "{XP3_PLUGIN_AUTHORIZATION_MESSAGE} Pass `{XP3_PLUGIN_AUTHORIZATION_FLAG}` to continue."
            );
        }
        Xp3PluginModule::from_json_file(&plugin_path)
            .with_context(|| format!("failed to load XP3 plugin {}", plugin_path.display()))?
            .xp3_options()
    } else {
        Xp3Options::default()
    };

    if let Some(path) = xp3_path {
        Xp3Archive::from_file_with_options(&path, options)
            .with_context(|| format!("failed to load XP3 archive {}", path.display()))?;
    }

    println!("check ok");
    Ok(())
}

fn xp3_path_from_input(input: &str) -> Result<PathBuf, String> {
    let cleaned = clean_path_input(input);
    if cleaned.is_empty() {
        return Err("Enter an XP3 path first.".to_owned());
    }

    let path = PathBuf::from(cleaned);
    if path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("xp3"))
    {
        Ok(path)
    } else {
        Err("The selected file is not an .xp3 archive.".to_owned())
    }
}

fn clean_path_input(input: &str) -> String {
    let mut value = input.trim().trim_matches(['"', '\'']).trim().to_owned();
    if let Some(rest) = value.strip_prefix("file:///") {
        value = rest.replace('/', "\\");
    } else if let Some(rest) = value.strip_prefix("file://") {
        value = rest.replace('/', "\\");
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_rejects_plugin_without_authorization() {
        let args = vec![
            OsString::from("--check"),
            OsString::from("--xp3-plugin"),
            OsString::from("plugin.json"),
        ];

        let error = format!("{:#}", dispatch(&args).unwrap_err());
        assert!(error.contains(XP3_PLUGIN_AUTHORIZATION_FLAG));
    }
}
