use anyhow::{Error, Result};
use avm::InstallTarget;
use clap::{Parser, Subcommand};
use semver::Version;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[clap(name = "avm", about = "Anchor version manager", version)]
pub struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[clap(about = "Use a specific version of Anchor")]
    Use {
        #[clap(value_parser = parse_version, required = false)]
        version: Option<Version>,
    },
    #[clap(about = "Install a version of Anchor")]
    Install {
        /// Anchor version or commit
        version: String,
        #[clap(short, long, required = false)]
        is_commit: bool,
        #[clap(long)]
        /// Flag to force installation even if the version
        /// is already installed
        force: bool,
    },
    #[clap(about = "Uninstall a version of Anchor")]
    Uninstall {
        #[clap(value_parser = parse_version)]
        version: Version,
    },
    #[clap(about = "List available versions of Anchor")]
    List {},
    #[clap(about = "Update to the latest Anchor version")]
    Update {},
}

// If `latest` is passed use the latest available version.
fn parse_version(version: &str) -> Result<Version, Error> {
    if version == "latest" {
        Ok(avm::get_latest_version())
    } else {
        Version::parse(version).map_err(|e| anyhow::anyhow!(e))
    }
}

fn parse_install_target(version_or_commit: String, is_commit: bool) -> InstallTarget {
    if is_commit {
        InstallTarget::Commit(version_or_commit)
    } else {
        InstallTarget::Version(parse_version(&version_or_commit).unwrap())
    }
}

pub fn entry(opts: Cli) -> Result<()> {
    match opts.command {
        Commands::Use { version } => avm::use_version(version),
        Commands::Install {
            version,
            is_commit,
            force,
        } => avm::install_anchor(parse_install_target(version, is_commit), force),
        Commands::Uninstall { version } => avm::uninstall_version(&version),
        Commands::List {} => avm::list_versions(),
        Commands::Update {} => avm::update(),
    }
}

fn main() -> Result<()> {
    // Make sure the user's home directory is setup with the paths required by AVM.
    avm::ensure_paths();

    let opt = Cli::parse();
    entry(opt)
}
