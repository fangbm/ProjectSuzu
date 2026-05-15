use std::{env, fs, path::PathBuf};

use anyhow::{bail, Context, Result};
use suzu_packer::{build_manifest, write_archive};

#[derive(Debug, Clone, PartialEq, Eq)]
struct PackerArgs {
    root: PathBuf,
    output: Option<PathBuf>,
    pack: Option<PathBuf>,
}

fn main() -> Result<()> {
    if env::args_os().len() == 1 {
        print_usage();
        pause_for_double_click();
        return Ok(());
    }

    let args = parse_args(env::args_os().skip(1))?;
    let manifest = build_manifest(&args.root)
        .with_context(|| format!("failed to build manifest for {}", args.root.display()))?;
    let json = serde_json::to_string_pretty(&manifest)?;

    if let Some(pack) = args.pack {
        write_archive(&args.root, manifest.clone(), &pack)
            .with_context(|| format!("failed to write archive {}", pack.display()))?;
    }

    if let Some(output) = args.output {
        if let Some(parent) = output
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        fs::write(&output, json)
            .with_context(|| format!("failed to write {}", output.display()))?;
    } else {
        println!("{json}");
    }

    Ok(())
}

fn print_usage() {
    println!("usage: suzu-packer <asset-root> [--output manifest.json] [--pack assets.suzupack]");
    println!();
    println!("examples:");
    println!("  suzu-packer examples\\hello-world --output target\\hello-world-assets.json");
    println!("  suzu-packer examples\\hello-world --pack target\\hello-world.suzupack");
}

fn pause_for_double_click() {
    #[cfg(windows)]
    {
        println!();
        println!("Press Enter to close...");
        let mut line = String::new();
        let _ = std::io::stdin().read_line(&mut line);
    }
}

fn parse_args<I>(args: I) -> Result<PackerArgs>
where
    I: IntoIterator,
    I::Item: Into<std::ffi::OsString>,
{
    let mut root = None;
    let mut output = None;
    let mut pack = None;
    let mut args = args.into_iter().map(Into::into).peekable();

    while let Some(arg) = args.next() {
        if arg == "--output" || arg == "-o" {
            let Some(value) = args.next() else {
                bail!("usage: suzu-packer <asset-root> [--output manifest.json] [--pack assets.suzupack]");
            };
            output = Some(PathBuf::from(value));
            continue;
        }
        if arg == "--pack" {
            let Some(value) = args.next() else {
                bail!("usage: suzu-packer <asset-root> [--output manifest.json] [--pack assets.suzupack]");
            };
            pack = Some(PathBuf::from(value));
            continue;
        }

        if root.is_some() {
            bail!(
                "usage: suzu-packer <asset-root> [--output manifest.json] [--pack assets.suzupack]"
            );
        }
        root = Some(PathBuf::from(arg));
    }

    let Some(root) = root else {
        bail!("usage: suzu-packer <asset-root> [--output manifest.json] [--pack assets.suzupack]");
    };

    Ok(PackerArgs { root, output, pack })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_root_and_output_args() {
        let args = parse_args(["assets", "--output", "manifest.json"]).unwrap();

        assert_eq!(args.root, PathBuf::from("assets"));
        assert_eq!(args.output, Some(PathBuf::from("manifest.json")));
        assert_eq!(args.pack, None);
    }
}
