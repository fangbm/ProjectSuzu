use std::{env, fs, path::PathBuf};

use anyhow::{bail, Context, Result};
use suzu_script::compile_script;

fn main() -> Result<()> {
    let input = input_path()?;
    let source = fs::read_to_string(&input)
        .with_context(|| format!("failed to read script {}", input.display()))?;
    let commands = compile_script(&source).context("failed to compile script")?;
    println!("{}", serde_json::to_string_pretty(&commands)?);
    Ok(())
}

fn input_path() -> Result<PathBuf> {
    let mut args = env::args_os().skip(1);
    let Some(input) = args.next() else {
        bail!("usage: suzu-compiler <input.szs>");
    };
    if args.next().is_some() {
        bail!("usage: suzu-compiler <input.szs>");
    }
    Ok(input.into())
}
