use anyhow::{anyhow, Result};
use clap::Clap;
use std::fs::{self};
use std::path::PathBuf;
use std::process::Stdio;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clap)]
#[clap(version = VERSION)]
pub struct Opts {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clap)]
pub enum Command {
    Init {},
    Use { version: String },
    Install { version: String },
    Uninstall { version: String },
}

pub fn entry(opts: Opts) -> Result<()> {
    match opts.command {
        Command::Use { version } => use_version(version),
        Command::Install { version } => install(version),
        Command::Uninstall { version } => uninstall(version),
    }
}

fn install(version: String) -> Result<()> {
    let exit = std::process::Command::new("cargo")
        .args(&[
            "install",
            "--git",
            "https://github.com/project-serum/anchor",
            "--tag",
            &version,
            "anchor-cli",
            "--locked",
        ])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| {
            anyhow::format_err!("Cargo install for {} failed: {}", version, e.to_string())
        })?;
    if !exit.status.success() {
        return Err(anyhow!("Failed to install {}", version));
    }

    Ok(())
}

fn uninstall(version: String) -> Result<()> {
    Ok(())
}

fn use_version(version: String) -> Result<()> {
    Ok(())
}

fn main() -> Result<()> {
    entry(Opts::parse())
}
