use crate::config::{
    AnchorPackage, Config, ConfigOverride, Manifest, ProgramDeployment, ProgramWorkspace, Test,
    WithPath,
};
use anchor_client::Cluster;
use anchor_lang::idl::{IdlAccount, IdlInstruction};
use anchor_lang::{AccountDeserialize, AnchorDeserialize, AnchorSerialize};
use anchor_syn::idl::Idl;
use anyhow::{anyhow, Context, Result};
use clap::Clap;
use flate2::read::GzDecoder;
use flate2::read::ZlibDecoder;
use flate2::write::{GzEncoder, ZlibEncoder};
use flate2::Compression;
use heck::SnakeCase;
use rand::rngs::OsRng;
use reqwest::blocking::multipart::{Form, Part};
use reqwest::blocking::Client;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_sdk::account_utils::StateMut;
use solana_sdk::bpf_loader_upgradeable::UpgradeableLoaderState;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signature::Signer;
use solana_sdk::sysvar;
use solana_sdk::transaction::Transaction;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};
use std::string::ToString;
use tar::Archive;

pub mod config;
pub mod template;

// Version of the docker image.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DOCKER_BUILDER_VERSION: &str = VERSION;

#[derive(Debug, Clap)]
#[clap(version = VERSION)]
pub struct Opts {
    #[clap(flatten)]
    pub cfg_override: ConfigOverride,
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clap)]
pub enum Command {
    /// Initializes a workspace.
    Init {
        name: String,
        #[clap(short, long)]
        javascript: bool,
    },
    /// Builds the workspace.
    Build {
        /// Output directory for the IDL.
        #[clap(short, long)]
        idl: Option<String>,
        /// Output directory for the TypeScript IDL.
        #[clap(short = 't', long)]
        idl_ts: Option<String>,
        /// True if the build artifact needs to be deterministic and verifiable.
        #[clap(short, long)]
        verifiable: bool,
        #[clap(short, long)]
        program_name: Option<String>,
        /// Version of the Solana toolchain to use. For --verifiable builds
        /// only.
        #[clap(short, long)]
        solana_version: Option<String>,
        /// Arguments to pass to the underlying `cargo build-bpf` command
        #[clap(
            required = false,
            takes_value = true,
            multiple_values = true,
            last = true
        )]
        cargo_args: Vec<String>,
    },
    /// Verifies the on-chain bytecode matches the locally compiled artifact.
    /// Run this command inside a program subdirectory, i.e., in the dir
    /// containing the program's Cargo.toml.
    Verify {
        /// The deployed program to compare against.
        program_id: Pubkey,
        #[clap(short, long)]
        program_name: Option<String>,
        /// Version of the Solana toolchain to use. For --verifiable builds
        /// only.
        #[clap(short, long)]
        solana_version: Option<String>,
        /// Arguments to pass to the underlying `cargo build-bpf` command.
        #[clap(
            required = false,
            takes_value = true,
            multiple_values = true,
            last = true
        )]
        cargo_args: Vec<String>,
    },
    /// Runs integration tests against a localnetwork.
    Test {
        /// Use this flag if you want to run tests against previously deployed
        /// programs.
        #[clap(long)]
        skip_deploy: bool,
        /// Flag to skip starting a local validator, if the configured cluster
        /// url is a localnet.
        #[clap(long)]
        skip_local_validator: bool,
        /// Flag to skip building the program in the workspace,
        /// use this to save time when running test and the program code is not altered.
        #[clap(long)]
        skip_build: bool,
        /// Flag to keep the local validator running after tests
        /// to be able to check the transactions.
        #[clap(long)]
        detach: bool,
        #[clap(multiple_values = true)]
        args: Vec<String>,
        /// Arguments to pass to the underlying `cargo build-bpf` command.
        #[clap(
            required = false,
            takes_value = true,
            multiple_values = true,
            last = true
        )]
        cargo_args: Vec<String>,
    },
    /// Creates a new program.
    New { name: String },
    /// Commands for interacting with interface definitions.
    Idl {
        #[clap(subcommand)]
        subcmd: IdlCommand,
    },
    /// Deploys each program in the workspace.
    Deploy {
        #[clap(short, long)]
        program_name: Option<String>,
    },
    /// Runs the deploy migration script.
    Migrate,
    /// Deploys, initializes an IDL, and migrates all in one command.
    /// Upgrades a single program. The configured wallet must be the upgrade
    /// authority.
    Upgrade {
        /// The program to upgrade.
        #[clap(short, long)]
        program_id: Pubkey,
        /// Filepath to the new program binary.
        program_filepath: String,
    },
    #[cfg(feature = "dev")]
    /// Runs an airdrop loop, continuously funding the configured wallet.
    Airdrop {
        #[clap(short, long)]
        url: Option<String>,
    },
    /// Cluster commands.
    Cluster {
        #[clap(subcommand)]
        subcmd: ClusterCommand,
    },
    /// Starts a node shell with an Anchor client setup according to the local
    /// config.
    Shell,
    /// Runs the script defined by the current workspace's Anchor.toml.
    Run {
        /// The name of the script to run.
        script: String,
    },
    /// Saves an api token from the registry locally.
    Login {
        /// API access token.
        token: String,
    },
    /// Publishes a verified build to the Anchor registry.
    Publish {
        /// The name of the program to publish.
        program: String,
        /// Arguments to pass to the underlying `cargo build-bpf` command.
        #[clap(
            required = false,
            takes_value = true,
            multiple_values = true,
            last = true
        )]
        cargo_args: Vec<String>,
    },
    /// Keypair commands.
    Keys {
        #[clap(subcommand)]
        subcmd: KeysCommand,
    },
    /// Localnet commands.
    Localnet {
        /// Flag to skip building the program in the workspace,
        /// use this to save time when running test and the program code is not altered.
        #[clap(long)]
        skip_build: bool,
        /// Use this flag if you want to run tests against previously deployed
        /// programs.
        #[clap(long)]
        skip_deploy: bool,
        /// Arguments to pass to the underlying `cargo build-bpf` command.
        #[clap(
            required = false,
            takes_value = true,
            multiple_values = true,
            last = true
        )]
        cargo_args: Vec<String>,
    },
}

#[derive(Debug, Clap)]
pub enum KeysCommand {
    List,
}

#[derive(Debug, Clap)]
pub enum IdlCommand {
    /// Initializes a program's IDL account. Can only be run once.
    Init {
        program_id: Pubkey,
        #[clap(short, long)]
        filepath: String,
    },
    /// Writes an IDL into a buffer account. This can be used with SetBuffer
    /// to perform an upgrade.
    WriteBuffer {
        program_id: Pubkey,
        #[clap(short, long)]
        filepath: String,
    },
    /// Sets a new IDL buffer for the program.
    SetBuffer {
        program_id: Pubkey,
        /// Address of the buffer account to set as the idl on the program.
        #[clap(short, long)]
        buffer: Pubkey,
    },
    /// Upgrades the IDL to the new file. An alias for first writing and then
    /// then setting the idl buffer account.
    Upgrade {
        program_id: Pubkey,
        #[clap(short, long)]
        filepath: String,
    },
    /// Sets a new authority on the IDL account.
    SetAuthority {
        /// The IDL account buffer to set the authority of. If none is given,
        /// then the canonical IDL account is used.
        address: Option<Pubkey>,
        /// Program to change the IDL authority.
        #[clap(short, long)]
        program_id: Pubkey,
        /// New authority of the IDL account.
        #[clap(short, long)]
        new_authority: Pubkey,
    },
    /// Command to remove the ability to modify the IDL account. This should
    /// likely be used in conjection with eliminating an "upgrade authority" on
    /// the program.
    EraseAuthority {
        #[clap(short, long)]
        program_id: Pubkey,
    },
    /// Outputs the authority for the IDL account.
    Authority {
        /// The program to view.
        program_id: Pubkey,
    },
    /// Parses an IDL from source.
    Parse {
        /// Path to the program's interface definition.
        #[clap(short, long)]
        file: String,
        /// Output file for the IDL (stdout if not specified).
        #[clap(short, long)]
        out: Option<String>,
        /// Output file for the TypeScript IDL.
        #[clap(short = 't', long)]
        out_ts: Option<String>,
    },
    /// Fetches an IDL for the given address from a cluster.
    /// The address can be a program, IDL account, or IDL buffer.
    Fetch {
        address: Pubkey,
        /// Output file for the idl (stdout if not specified).
        #[clap(short, long)]
        out: Option<String>,
    },
}

#[derive(Debug, Clap)]
pub enum ClusterCommand {
    /// Prints common cluster urls.
    List,
}

pub fn entry(opts: Opts) -> Result<()> {
    match opts.command {
        Command::Init { name, javascript } => init(&opts.cfg_override, name, javascript),
        Command::New { name } => new(&opts.cfg_override, name),
        Command::Build {
            idl,
            idl_ts,
            verifiable,
            program_name,
            solana_version,
            cargo_args,
        } => build(
            &opts.cfg_override,
            idl,
            idl_ts,
            verifiable,
            program_name,
            solana_version,
            None,
            None,
            cargo_args,
        ),
        Command::Verify {
            program_id,
            program_name,
            solana_version,
            cargo_args,
        } => verify(
            &opts.cfg_override,
            program_id,
            program_name,
            solana_version,
            cargo_args,
        ),
        Command::Deploy { program_name } => deploy(&opts.cfg_override, program_name),
        Command::Upgrade {
            program_id,
            program_filepath,
        } => upgrade(&opts.cfg_override, program_id, program_filepath),
        Command::Idl { subcmd } => idl(&opts.cfg_override, subcmd),
        Command::Migrate => migrate(&opts.cfg_override),
        Command::Test {
            skip_deploy,
            skip_local_validator,
            skip_build,
            detach,
            args,
            cargo_args,
        } => test(
            &opts.cfg_override,
            skip_deploy,
            skip_local_validator,
            skip_build,
            detach,
            args,
            cargo_args,
        ),
        #[cfg(feature = "dev")]
        Command::Airdrop => airdrop(cfg_override),
        Command::Cluster { subcmd } => cluster(subcmd),
        Command::Shell => shell(&opts.cfg_override),
        Command::Run { script } => run(&opts.cfg_override, script),
        Command::Login { token } => login(&opts.cfg_override, token),
        Command::Publish {
            program,
            cargo_args,
        } => publish(&opts.cfg_override, program, cargo_args),
        Command::Keys { subcmd } => keys(&opts.cfg_override, subcmd),
        Command::Localnet {
            skip_build,
            skip_deploy,
            cargo_args,
        } => localnet(&opts.cfg_override, skip_build, skip_deploy, cargo_args),
    }
}

fn init(cfg_override: &ConfigOverride, name: String, javascript: bool) -> Result<()> {
    if Config::discover(cfg_override)?.is_some() {
        return Err(anyhow!("Workspace already initialized"));
    }

    fs::create_dir(name.clone())?;
    std::env::set_current_dir(&name)?;
    fs::create_dir("app")?;

    let mut cfg = Config::default();
    cfg.scripts.insert(
        "test".to_owned(),
        if javascript {
            "yarn run mocha -t 1000000 tests/"
        } else {
            "yarn run ts-mocha -p ./tsconfig.json -t 1000000 tests/**/*.ts"
        }
        .to_owned(),
    );
    let mut localnet = BTreeMap::new();
    localnet.insert(
        name.to_snake_case(),
        ProgramDeployment {
            address: template::default_program_id(),
            path: None,
            idl: None,
        },
    );
    cfg.programs.insert(Cluster::Localnet, localnet);
    let toml = cfg.to_string();
    let mut file = File::create("Anchor.toml")?;
    file.write_all(toml.as_bytes())?;

    // Build virtual manifest.
    let mut virt_manifest = File::create("Cargo.toml")?;
    virt_manifest.write_all(template::virtual_manifest().as_bytes())?;

    // Initialize .gitignore file
    let mut virt_manifest = File::create(".gitignore")?;
    virt_manifest.write_all(template::git_ignore().as_bytes())?;

    // Build the program.
    fs::create_dir("programs")?;

    new_program(&name)?;

    // Build the test suite.
    fs::create_dir("tests")?;
    // Build the migrations directory.
    fs::create_dir("migrations")?;

    if javascript {
        // Build javascript config
        let mut package_json = File::create("package.json")?;
        package_json.write_all(template::package_json().as_bytes())?;

        let mut mocha = File::create(&format!("tests/{}.js", name))?;
        mocha.write_all(template::mocha(&name).as_bytes())?;

        let mut deploy = File::create("migrations/deploy.js")?;
        deploy.write_all(template::deploy_script().as_bytes())?;
    } else {
        // Build typescript config
        let mut ts_config = File::create("tsconfig.json")?;
        ts_config.write_all(template::ts_config().as_bytes())?;

        let mut ts_package_json = File::create("package.json")?;
        ts_package_json.write_all(template::ts_package_json().as_bytes())?;

        let mut deploy = File::create("migrations/deploy.ts")?;
        deploy.write_all(template::ts_deploy_script().as_bytes())?;

        let mut mocha = File::create(&format!("tests/{}.ts", name))?;
        mocha.write_all(template::ts_mocha(&name).as_bytes())?;
    }

    // Install node modules.
    let yarn_result = std::process::Command::new("yarn")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| anyhow::format_err!("yarn install failed: {}", e.to_string()))?;
    if !yarn_result.status.success() {
        println!("Failed yarn install will attempt to npm install");
        std::process::Command::new("npm")
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .map_err(|e| anyhow::format_err!("npm install failed: {}", e.to_string()))?;
        println!("Failed to install node dependencies")
    }

    println!("{} initialized", name);

    Ok(())
}

// Creates a new program crate in the `programs/<name>` directory.
fn new(cfg_override: &ConfigOverride, name: String) -> Result<()> {
    with_workspace(cfg_override, |cfg| {
        match cfg.path().parent() {
            None => {
                println!("Unable to make new program");
            }
            Some(parent) => {
                std::env::set_current_dir(&parent)?;
                new_program(&name)?;
                println!("Created new program.");
            }
        };
        Ok(())
    })
}

// Creates a new program crate in the current directory with `name`.
fn new_program(name: &str) -> Result<()> {
    fs::create_dir(&format!("programs/{}", name))?;
    fs::create_dir(&format!("programs/{}/src/", name))?;
    let mut cargo_toml = File::create(&format!("programs/{}/Cargo.toml", name))?;
    cargo_toml.write_all(template::cargo_toml(name).as_bytes())?;
    let mut xargo_toml = File::create(&format!("programs/{}/Xargo.toml", name))?;
    xargo_toml.write_all(template::xargo_toml().as_bytes())?;
    let mut lib_rs = File::create(&format!("programs/{}/src/lib.rs", name))?;
    lib_rs.write_all(template::lib_rs(name).as_bytes())?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn build(
    cfg_override: &ConfigOverride,
    idl: Option<String>,
    idl_ts: Option<String>,
    verifiable: bool,
    program_name: Option<String>,
    solana_version: Option<String>,
    stdout: Option<File>, // Used for the package registry server.
    stderr: Option<File>, // Used for the package registry server.
    cargo_args: Vec<String>,
) -> Result<()> {
    // Change to the workspace member directory, if needed.
    if let Some(program_name) = program_name.as_ref() {
        cd_member(cfg_override, program_name)?;
    }

    let cfg = Config::discover(cfg_override)?.expect("Not in workspace.");
    let cfg_parent = cfg.path().parent().expect("Invalid Anchor.toml");

    let cargo = Manifest::discover()?;

    let idl_out = match idl {
        Some(idl) => Some(PathBuf::from(idl)),
        None => Some(cfg_parent.join("target/idl")),
    };
    fs::create_dir_all(idl_out.as_ref().unwrap())?;

    let idl_ts_out = match idl_ts {
        Some(idl_ts) => Some(PathBuf::from(idl_ts)),
        None => Some(cfg_parent.join("target/types")),
    };
    fs::create_dir_all(idl_ts_out.as_ref().unwrap())?;

    let solana_version = match solana_version.is_some() {
        true => solana_version,
        false => cfg.solana_version.clone(),
    };

    match cargo {
        // No Cargo.toml so build the entire workspace.
        None => build_all(
            &cfg,
            cfg.path(),
            idl_out,
            idl_ts_out,
            verifiable,
            solana_version,
            stdout,
            stderr,
            cargo_args,
        )?,
        // If the Cargo.toml is at the root, build the entire workspace.
        Some(cargo) if cargo.path().parent() == cfg.path().parent() => build_all(
            &cfg,
            cfg.path(),
            idl_out,
            idl_ts_out,
            verifiable,
            solana_version,
            stdout,
            stderr,
            cargo_args,
        )?,
        // Cargo.toml represents a single package. Build it.
        Some(cargo) => build_cwd(
            &cfg,
            cargo.path().to_path_buf(),
            idl_out,
            idl_ts_out,
            verifiable,
            solana_version,
            stdout,
            stderr,
            cargo_args,
        )?,
    }

    set_workspace_dir_or_exit();

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn build_all(
    cfg: &WithPath<Config>,
    cfg_path: &Path,
    idl_out: Option<PathBuf>,
    idl_ts_out: Option<PathBuf>,
    verifiable: bool,
    solana_version: Option<String>,
    stdout: Option<File>, // Used for the package registry server.
    stderr: Option<File>, // Used for the package registry server.
    cargo_args: Vec<String>,
) -> Result<()> {
    let cur_dir = std::env::current_dir()?;
    let r = match cfg_path.parent() {
        None => Err(anyhow!("Invalid Anchor.toml at {}", cfg_path.display())),
        Some(_parent) => {
            for p in cfg.get_program_list()? {
                build_cwd(
                    cfg,
                    p.join("Cargo.toml"),
                    idl_out.clone(),
                    idl_ts_out.clone(),
                    verifiable,
                    solana_version.clone(),
                    stdout.as_ref().map(|f| f.try_clone()).transpose()?,
                    stderr.as_ref().map(|f| f.try_clone()).transpose()?,
                    cargo_args.clone(),
                )?;
            }
            Ok(())
        }
    };
    std::env::set_current_dir(cur_dir)?;
    r
}

// Runs the build command outside of a workspace.
#[allow(clippy::too_many_arguments)]
fn build_cwd(
    cfg: &WithPath<Config>,
    cargo_toml: PathBuf,
    idl_out: Option<PathBuf>,
    idl_ts_out: Option<PathBuf>,
    verifiable: bool,
    solana_version: Option<String>,
    stdout: Option<File>,
    stderr: Option<File>,
    cargo_args: Vec<String>,
) -> Result<()> {
    match cargo_toml.parent() {
        None => return Err(anyhow!("Unable to find parent")),
        Some(p) => std::env::set_current_dir(&p)?,
    };
    match verifiable {
        false => _build_cwd(idl_out, idl_ts_out, cargo_args),
        true => build_cwd_verifiable(cfg, cargo_toml, solana_version, stdout, stderr),
    }
}

// Builds an anchor program in a docker image and copies the build artifacts
// into the `target/` directory.
fn build_cwd_verifiable(
    cfg: &WithPath<Config>,
    cargo_toml: PathBuf,
    solana_version: Option<String>,
    stdout: Option<File>,
    stderr: Option<File>,
) -> Result<()> {
    // Create output dirs.
    let workspace_dir = cfg.path().parent().unwrap().canonicalize()?;
    fs::create_dir_all(workspace_dir.join("target/verifiable"))?;
    fs::create_dir_all(workspace_dir.join("target/idl"))?;
    fs::create_dir_all(workspace_dir.join("target/types"))?;

    let container_name = "anchor-program";

    // Build the binary in docker.
    let result = docker_build(
        cfg,
        container_name,
        cargo_toml,
        solana_version,
        stdout,
        stderr,
    );

    // Wipe the generated docker-target dir.
    println!("Cleaning up the docker target directory");
    let exit = std::process::Command::new("docker")
        .args(&[
            "exec",
            container_name,
            "rm",
            "-rf",
            "/workdir/docker-target",
        ])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| anyhow::format_err!("Docker rm docker-target failed: {}", e.to_string()))?;
    if !exit.status.success() {
        return Err(anyhow!("Failed to build program"));
    }

    // Remove the docker image.
    println!("Removing the docker image");
    let exit = std::process::Command::new("docker")
        .args(&["rm", "-f", container_name])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| anyhow::format_err!("{}", e.to_string()))?;
    if !exit.status.success() {
        println!("Unable to remove docker container");
        std::process::exit(exit.status.code().unwrap_or(1));
    }

    // Build the idl.
    println!("Extracting the IDL");
    if let Ok(Some(idl)) = extract_idl("src/lib.rs") {
        // Write out the JSON file.
        println!("Writing the IDL file");
        let out_file = workspace_dir.join(format!("target/idl/{}.json", idl.name));
        write_idl(&idl, OutFile::File(out_file))?;

        // Write out the TypeScript type.
        println!("Writing the .ts file");
        let ts_file = workspace_dir.join(format!("target/types/{}.ts", idl.name));
        fs::write(&ts_file, template::idl_ts(&idl)?)?;
    }
    println!("Build success");

    result
}

fn docker_build(
    cfg: &WithPath<Config>,
    container_name: &str,
    cargo_toml: PathBuf,
    solana_version: Option<String>,
    stdout: Option<File>,
    stderr: Option<File>,
) -> Result<()> {
    let binary_name = Manifest::from_path(&cargo_toml)?.lib_name()?;

    // Docker vars.
    let image_name = cfg.docker();
    let volume_mount = format!(
        "{}:/workdir",
        cfg.path().parent().unwrap().canonicalize()?.display()
    );
    println!("Using image {:?}", image_name);

    // Start the docker image running detached in the background.
    println!("Run docker image");
    let exit = std::process::Command::new("docker")
        .args(&[
            "run",
            "-it",
            "-d",
            "--name",
            container_name,
            "--env",
            "CARGO_TARGET_DIR=/workdir/docker-target",
            "-v",
            &volume_mount,
            &image_name,
            "bash",
        ])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| anyhow::format_err!("Docker build failed: {}", e.to_string()))?;
    if !exit.status.success() {
        return Err(anyhow!("Failed to build program"));
    }

    // Set the solana version in the container, if given. Otherwise use the
    // default.
    if let Some(solana_version) = solana_version {
        println!("Using solana version: {}", solana_version);

        // Fetch the installer.
        let exit = std::process::Command::new("docker")
            .args(&[
                "exec",
                container_name,
                "curl",
                "-sSfL",
                &format!("https://release.solana.com/v{0}/install", solana_version,),
                "-o",
                "solana_installer.sh",
            ])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .map_err(|e| anyhow!("Failed to set solana version: {:?}", e))?;
        if !exit.status.success() {
            return Err(anyhow!("Failed to set solana version"));
        }

        // Run the installer.
        let exit = std::process::Command::new("docker")
            .args(&["exec", container_name, "sh", "solana_installer.sh"])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .map_err(|e| anyhow!("Failed to set solana version: {:?}", e))?;
        if !exit.status.success() {
            return Err(anyhow!("Failed to set solana version"));
        }

        // Remove the installer.
        let exit = std::process::Command::new("docker")
            .args(&["exec", container_name, "rm", "-f", "solana_installer.sh"])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .map_err(|e| anyhow!("Failed to remove installer: {:?}", e))?;
        if !exit.status.success() {
            return Err(anyhow!("Failed to remove installer"));
        }
    }

    let manifest_path = pathdiff::diff_paths(
        cargo_toml.canonicalize()?,
        cfg.path().parent().unwrap().canonicalize()?,
    )
    .ok_or_else(|| anyhow!("Unable to diff paths"))?;
    println!(
        "Building {} manifest: {:?}",
        binary_name,
        manifest_path.display().to_string()
    );

    // Execute the build.
    let exit = std::process::Command::new("docker")
        .args(&[
            "exec",
            container_name,
            "cargo",
            "build-bpf",
            "--manifest-path",
            &manifest_path.display().to_string(),
        ])
        .stdout(match stdout {
            None => Stdio::inherit(),
            Some(f) => f.into(),
        })
        .stderr(match stderr {
            None => Stdio::inherit(),
            Some(f) => f.into(),
        })
        .output()
        .map_err(|e| anyhow::format_err!("Docker build failed: {}", e.to_string()))?;
    if !exit.status.success() {
        return Err(anyhow!("Failed to build program"));
    }

    // Copy the binary out of the docker image.
    println!("Copying out the build artifacts");
    let out_file = cfg
        .path()
        .parent()
        .unwrap()
        .canonicalize()?
        .join(format!("target/verifiable/{}.so", binary_name))
        .display()
        .to_string();

    // This requires the target directory of any built program to be located at
    // the root of the workspace.
    let bin_artifact = format!(
        "{}:/workdir/docker-target/deploy/{}.so",
        container_name, binary_name
    );
    let exit = std::process::Command::new("docker")
        .args(&["cp", &bin_artifact, &out_file])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| anyhow::format_err!("{}", e.to_string()))?;
    if !exit.status.success() {
        return Err(anyhow!(
            "Failed to copy binary out of docker. Is the target directory set correctly?"
        ));
    }

    // Done.
    Ok(())
}

fn _build_cwd(
    idl_out: Option<PathBuf>,
    idl_ts_out: Option<PathBuf>,
    cargo_args: Vec<String>,
) -> Result<()> {
    let exit = std::process::Command::new("cargo")
        .arg("build-bpf")
        .args(cargo_args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| anyhow::format_err!("{}", e.to_string()))?;
    if !exit.status.success() {
        std::process::exit(exit.status.code().unwrap_or(1));
    }

    // Always assume idl is located at src/lib.rs.
    if let Some(idl) = extract_idl("src/lib.rs")? {
        // JSON out path.
        let out = match idl_out {
            None => PathBuf::from(".").join(&idl.name).with_extension("json"),
            Some(o) => PathBuf::from(&o.join(&idl.name).with_extension("json")),
        };
        // TS out path.
        let ts_out = match idl_ts_out {
            None => PathBuf::from(".").join(&idl.name).with_extension("ts"),
            Some(o) => PathBuf::from(&o.join(&idl.name).with_extension("ts")),
        };

        // Write out the JSON file.
        write_idl(&idl, OutFile::File(out))?;

        // Write out the TypeScript type.
        fs::write(ts_out, template::idl_ts(&idl)?)?;
    }

    Ok(())
}

fn verify(
    cfg_override: &ConfigOverride,
    program_id: Pubkey,
    program_name: Option<String>,
    solana_version: Option<String>,
    cargo_args: Vec<String>,
) -> Result<()> {
    // Change to the workspace member directory, if needed.
    if let Some(program_name) = program_name.as_ref() {
        cd_member(cfg_override, program_name)?;
    }

    // Proceed with the command.
    let cfg = Config::discover(cfg_override)?.expect("Not in workspace.");
    let cargo = Manifest::discover()?.ok_or_else(|| anyhow!("Cargo.toml not found"))?;

    // Build the program we want to verify.
    let cur_dir = std::env::current_dir()?;
    build(
        cfg_override,
        None,
        None,
        true,
        None,
        match solana_version.is_some() {
            true => solana_version,
            false => cfg.solana_version.clone(),
        },
        None,
        None,
        cargo_args,
    )?;
    std::env::set_current_dir(&cur_dir)?;

    // Verify binary.
    let binary_name = cargo.lib_name()?;
    let bin_path = cfg
        .path()
        .parent()
        .ok_or_else(|| anyhow!("Unable to find workspace root"))?
        .join("target/verifiable/")
        .join(format!("{}.so", binary_name));

    let url = cluster_url(&cfg);
    let bin_ver = verify_bin(program_id, &bin_path, &url)?;
    if !bin_ver.is_verified {
        println!("Error: Binaries don't match");
        std::process::exit(1);
    }

    // Verify IDL (only if it's not a buffer account).
    if let Some(local_idl) = extract_idl("src/lib.rs")? {
        if bin_ver.state != BinVerificationState::Buffer {
            let deployed_idl = fetch_idl(cfg_override, program_id)?;
            if local_idl != deployed_idl {
                println!("Error: IDLs don't match");
                std::process::exit(1);
            }
        }
    }

    println!("{} is verified.", program_id);

    Ok(())
}

fn cd_member(cfg_override: &ConfigOverride, program_name: &str) -> Result<()> {
    // Change directories to the given `program_name`, if given.
    let cfg = Config::discover(cfg_override)?.expect("Not in workspace.");

    for program in cfg.read_all_programs()? {
        let cargo_toml = program.path.join("Cargo.toml");
        if !cargo_toml.exists() {
            return Err(anyhow!(
                "Did not find Cargo.toml at the path: {}",
                program.path.display()
            ));
        }
        let p_lib_name = Manifest::from_path(&cargo_toml)?.lib_name()?;
        if program_name == p_lib_name {
            std::env::set_current_dir(&program.path)?;
            return Ok(());
        }
    }
    return Err(anyhow!("{} is not part of the workspace", program_name,));
}

pub fn verify_bin(program_id: Pubkey, bin_path: &Path, cluster: &str) -> Result<BinVerification> {
    let client = RpcClient::new(cluster.to_string());

    // Get the deployed build artifacts.
    let (deployed_bin, state) = {
        let account = client
            .get_account_with_commitment(&program_id, CommitmentConfig::default())?
            .value
            .map_or(Err(anyhow!("Account not found")), Ok)?;
        match account.state()? {
            UpgradeableLoaderState::Program {
                programdata_address,
            } => {
                let account = client
                    .get_account_with_commitment(&programdata_address, CommitmentConfig::default())?
                    .value
                    .map_or(Err(anyhow!("Account not found")), Ok)?;
                let bin = account.data
                    [UpgradeableLoaderState::programdata_data_offset().unwrap_or(0)..]
                    .to_vec();

                if let UpgradeableLoaderState::ProgramData {
                    slot,
                    upgrade_authority_address,
                } = account.state()?
                {
                    let state = BinVerificationState::ProgramData {
                        slot,
                        upgrade_authority_address,
                    };
                    (bin, state)
                } else {
                    return Err(anyhow!("Expected program data"));
                }
            }
            UpgradeableLoaderState::Buffer { .. } => {
                let offset = UpgradeableLoaderState::buffer_data_offset().unwrap_or(0);
                (
                    account.data[offset..].to_vec(),
                    BinVerificationState::Buffer,
                )
            }
            _ => return Err(anyhow!("Invalid program id")),
        }
    };
    let mut local_bin = {
        let mut f = File::open(bin_path)?;
        let mut contents = vec![];
        f.read_to_end(&mut contents)?;
        contents
    };

    // The deployed program probably has zero bytes appended. The default is
    // 2x the binary size in case of an upgrade.
    if local_bin.len() < deployed_bin.len() {
        local_bin.append(&mut vec![0; deployed_bin.len() - local_bin.len()]);
    }

    // Finally, check the bytes.
    let is_verified = local_bin == deployed_bin;

    Ok(BinVerification { state, is_verified })
}

#[derive(PartialEq)]
pub struct BinVerification {
    pub state: BinVerificationState,
    pub is_verified: bool,
}

#[derive(PartialEq)]
pub enum BinVerificationState {
    Buffer,
    ProgramData {
        slot: u64,
        upgrade_authority_address: Option<Pubkey>,
    },
}

// Fetches an IDL for the given program_id.
fn fetch_idl(cfg_override: &ConfigOverride, idl_addr: Pubkey) -> Result<Idl> {
    let cfg = Config::discover(cfg_override)?.expect("Inside a workspace");
    let url = cluster_url(&cfg);
    let client = RpcClient::new(url);

    let mut account = client
        .get_account_with_commitment(&idl_addr, CommitmentConfig::processed())?
        .value
        .map_or(Err(anyhow!("Account not found")), Ok)?;

    if account.executable {
        let idl_addr = IdlAccount::address(&idl_addr);
        account = client
            .get_account_with_commitment(&idl_addr, CommitmentConfig::processed())?
            .value
            .map_or(Err(anyhow!("Account not found")), Ok)?;
    }

    // Cut off account discriminator.
    let mut d: &[u8] = &account.data[8..];
    let idl_account: IdlAccount = AnchorDeserialize::deserialize(&mut d)?;

    let mut z = ZlibDecoder::new(&idl_account.data[..]);
    let mut s = Vec::new();
    z.read_to_end(&mut s)?;
    serde_json::from_slice(&s[..]).map_err(Into::into)
}

fn extract_idl(file: &str) -> Result<Option<Idl>> {
    let file = shellexpand::tilde(file);
    anchor_syn::idl::file::parse(&*file)
}

fn idl(cfg_override: &ConfigOverride, subcmd: IdlCommand) -> Result<()> {
    match subcmd {
        IdlCommand::Init {
            program_id,
            filepath,
        } => idl_init(cfg_override, program_id, filepath),
        IdlCommand::WriteBuffer {
            program_id,
            filepath,
        } => idl_write_buffer(cfg_override, program_id, filepath).map(|_| ()),
        IdlCommand::SetBuffer { program_id, buffer } => {
            idl_set_buffer(cfg_override, program_id, buffer)
        }
        IdlCommand::Upgrade {
            program_id,
            filepath,
        } => idl_upgrade(cfg_override, program_id, filepath),
        IdlCommand::SetAuthority {
            program_id,
            address,
            new_authority,
        } => idl_set_authority(cfg_override, program_id, address, new_authority),
        IdlCommand::EraseAuthority { program_id } => idl_erase_authority(cfg_override, program_id),
        IdlCommand::Authority { program_id } => idl_authority(cfg_override, program_id),
        IdlCommand::Parse { file, out, out_ts } => idl_parse(file, out, out_ts),
        IdlCommand::Fetch { address, out } => idl_fetch(cfg_override, address, out),
    }
}

fn idl_init(cfg_override: &ConfigOverride, program_id: Pubkey, idl_filepath: String) -> Result<()> {
    with_workspace(cfg_override, |cfg| {
        let keypair = cfg.provider.wallet.to_string();

        let bytes = fs::read(idl_filepath)?;
        let idl: Idl = serde_json::from_reader(&*bytes)?;

        let idl_address = create_idl_account(cfg, &keypair, &program_id, &idl)?;

        println!("Idl account created: {:?}", idl_address);
        Ok(())
    })
}

fn idl_write_buffer(
    cfg_override: &ConfigOverride,
    program_id: Pubkey,
    idl_filepath: String,
) -> Result<Pubkey> {
    with_workspace(cfg_override, |cfg| {
        let keypair = cfg.provider.wallet.to_string();

        let bytes = fs::read(idl_filepath)?;
        let idl: Idl = serde_json::from_reader(&*bytes)?;

        let idl_buffer = create_idl_buffer(cfg, &keypair, &program_id, &idl)?;
        idl_write(cfg, &program_id, &idl, idl_buffer)?;

        println!("Idl buffer created: {:?}", idl_buffer);

        Ok(idl_buffer)
    })
}

fn idl_set_buffer(cfg_override: &ConfigOverride, program_id: Pubkey, buffer: Pubkey) -> Result<()> {
    with_workspace(cfg_override, |cfg| {
        let keypair = solana_sdk::signature::read_keypair_file(&cfg.provider.wallet.to_string())
            .map_err(|_| anyhow!("Unable to read keypair file"))?;
        let url = cluster_url(cfg);
        let client = RpcClient::new(url);

        // Instruction to set the buffer onto the IdlAccount.
        let set_buffer_ix = {
            let accounts = vec![
                AccountMeta::new(buffer, false),
                AccountMeta::new(IdlAccount::address(&program_id), false),
                AccountMeta::new(keypair.pubkey(), true),
            ];
            let mut data = anchor_lang::idl::IDL_IX_TAG.to_le_bytes().to_vec();
            data.append(&mut IdlInstruction::SetBuffer.try_to_vec()?);
            Instruction {
                program_id,
                accounts,
                data,
            }
        };

        // Build the transaction.
        let (recent_hash, _fee_calc) = client.get_recent_blockhash()?;
        let tx = Transaction::new_signed_with_payer(
            &[set_buffer_ix],
            Some(&keypair.pubkey()),
            &[&keypair],
            recent_hash,
        );

        // Send the transaction.
        client.send_and_confirm_transaction_with_spinner_and_config(
            &tx,
            CommitmentConfig::confirmed(),
            RpcSendTransactionConfig {
                skip_preflight: true,
                ..RpcSendTransactionConfig::default()
            },
        )?;

        Ok(())
    })
}

fn idl_upgrade(
    cfg_override: &ConfigOverride,
    program_id: Pubkey,
    idl_filepath: String,
) -> Result<()> {
    let buffer = idl_write_buffer(cfg_override, program_id, idl_filepath)?;
    idl_set_buffer(cfg_override, program_id, buffer)
}

fn idl_authority(cfg_override: &ConfigOverride, program_id: Pubkey) -> Result<()> {
    with_workspace(cfg_override, |cfg| {
        let url = cluster_url(cfg);
        let client = RpcClient::new(url);
        let idl_address = {
            let account = client
                .get_account_with_commitment(&program_id, CommitmentConfig::processed())?
                .value
                .map_or(Err(anyhow!("Account not found")), Ok)?;
            if account.executable {
                IdlAccount::address(&program_id)
            } else {
                program_id
            }
        };

        let account = client.get_account(&idl_address)?;
        let mut data: &[u8] = &account.data;
        let idl_account: IdlAccount = AccountDeserialize::try_deserialize(&mut data)?;

        println!("{:?}", idl_account.authority);

        Ok(())
    })
}

fn idl_set_authority(
    cfg_override: &ConfigOverride,
    program_id: Pubkey,
    address: Option<Pubkey>,
    new_authority: Pubkey,
) -> Result<()> {
    with_workspace(cfg_override, |cfg| {
        // Misc.
        let idl_address = match address {
            None => IdlAccount::address(&program_id),
            Some(addr) => addr,
        };
        let keypair = solana_sdk::signature::read_keypair_file(&cfg.provider.wallet.to_string())
            .map_err(|_| anyhow!("Unable to read keypair file"))?;
        let url = cluster_url(cfg);
        let client = RpcClient::new(url);

        // Instruction data.
        let data =
            serialize_idl_ix(anchor_lang::idl::IdlInstruction::SetAuthority { new_authority })?;

        // Instruction accounts.
        let accounts = vec![
            AccountMeta::new(idl_address, false),
            AccountMeta::new_readonly(keypair.pubkey(), true),
        ];

        // Instruction.
        let ix = Instruction {
            program_id,
            accounts,
            data,
        };
        // Send transaction.
        let (recent_hash, _fee_calc) = client.get_recent_blockhash()?;
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&keypair.pubkey()),
            &[&keypair],
            recent_hash,
        );
        client.send_and_confirm_transaction_with_spinner_and_config(
            &tx,
            CommitmentConfig::confirmed(),
            RpcSendTransactionConfig {
                skip_preflight: true,
                ..RpcSendTransactionConfig::default()
            },
        )?;

        println!("Authority update complete.");

        Ok(())
    })
}

fn idl_erase_authority(cfg_override: &ConfigOverride, program_id: Pubkey) -> Result<()> {
    println!("Are you sure you want to erase the IDL authority: [y/n]");

    let stdin = std::io::stdin();
    let mut stdin_lines = stdin.lock().lines();
    let input = stdin_lines.next().unwrap().unwrap();
    if input != "y" {
        println!("Not erasing.");
        return Ok(());
    }

    // Program will treat the zero authority as erased.
    let new_authority = Pubkey::new_from_array([0u8; 32]);
    idl_set_authority(cfg_override, program_id, None, new_authority)?;

    Ok(())
}

// Write the idl to the account buffer, chopping up the IDL into pieces
// and sending multiple transactions in the event the IDL doesn't fit into
// a single transaction.
fn idl_write(cfg: &Config, program_id: &Pubkey, idl: &Idl, idl_address: Pubkey) -> Result<()> {
    // Remove the metadata before deploy.
    let mut idl = idl.clone();
    idl.metadata = None;

    // Misc.
    let keypair = solana_sdk::signature::read_keypair_file(&cfg.provider.wallet.to_string())
        .map_err(|_| anyhow!("Unable to read keypair file"))?;
    let url = cluster_url(cfg);
    let client = RpcClient::new(url);

    // Serialize and compress the idl.
    let idl_data = {
        let json_bytes = serde_json::to_vec(&idl)?;
        let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
        e.write_all(&json_bytes)?;
        e.finish()?
    };

    const MAX_WRITE_SIZE: usize = 1000;
    let mut offset = 0;
    while offset < idl_data.len() {
        // Instruction data.
        let data = {
            let start = offset;
            let end = std::cmp::min(offset + MAX_WRITE_SIZE, idl_data.len());
            serialize_idl_ix(anchor_lang::idl::IdlInstruction::Write {
                data: idl_data[start..end].to_vec(),
            })?
        };
        // Instruction accounts.
        let accounts = vec![
            AccountMeta::new(idl_address, false),
            AccountMeta::new_readonly(keypair.pubkey(), true),
        ];
        // Instruction.
        let ix = Instruction {
            program_id: *program_id,
            accounts,
            data,
        };
        // Send transaction.
        let (recent_hash, _fee_calc) = client.get_recent_blockhash()?;
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&keypair.pubkey()),
            &[&keypair],
            recent_hash,
        );
        client.send_and_confirm_transaction_with_spinner_and_config(
            &tx,
            CommitmentConfig::confirmed(),
            RpcSendTransactionConfig {
                skip_preflight: true,
                ..RpcSendTransactionConfig::default()
            },
        )?;
        offset += MAX_WRITE_SIZE;
    }
    Ok(())
}

fn idl_parse(file: String, out: Option<String>, out_ts: Option<String>) -> Result<()> {
    let idl = extract_idl(&file)?.ok_or_else(|| anyhow!("IDL not parsed"))?;
    let out = match out {
        None => OutFile::Stdout,
        Some(out) => OutFile::File(PathBuf::from(out)),
    };
    write_idl(&idl, out)?;

    // Write out the TypeScript IDL.
    if let Some(out) = out_ts {
        fs::write(out, template::idl_ts(&idl)?)?;
    }

    Ok(())
}

fn idl_fetch(cfg_override: &ConfigOverride, address: Pubkey, out: Option<String>) -> Result<()> {
    let idl = fetch_idl(cfg_override, address)?;
    let out = match out {
        None => OutFile::Stdout,
        Some(out) => OutFile::File(PathBuf::from(out)),
    };
    write_idl(&idl, out)
}

fn write_idl(idl: &Idl, out: OutFile) -> Result<()> {
    let idl_json = serde_json::to_string_pretty(idl)?;
    match out {
        OutFile::Stdout => println!("{}", idl_json),
        OutFile::File(out) => fs::write(out, idl_json)?,
    };

    Ok(())
}

enum OutFile {
    Stdout,
    File(PathBuf),
}

// Builds, deploys, and tests all workspace programs in a single command.
fn test(
    cfg_override: &ConfigOverride,
    skip_deploy: bool,
    skip_local_validator: bool,
    skip_build: bool,
    detach: bool,
    extra_args: Vec<String>,
    cargo_args: Vec<String>,
) -> Result<()> {
    with_workspace(cfg_override, |cfg| {
        // Build if needed.
        if !skip_build {
            build(
                cfg_override,
                None,
                None,
                false,
                None,
                None,
                None,
                None,
                cargo_args,
            )?;
        }

        // Run the deploy against the cluster in two cases:
        //
        // 1. The cluster is not localnet.
        // 2. The cluster is localnet, but we're not booting a local validator.
        //
        // In either case, skip the deploy if the user specifies.
        let is_localnet = cfg.provider.cluster == Cluster::Localnet;
        if (!is_localnet || skip_local_validator) && !skip_deploy {
            deploy(cfg_override, None)?;
        }
        // Start local test validator, if needed.
        let mut validator_handle = None;
        if is_localnet && (!skip_local_validator) {
            let flags = match skip_deploy {
                true => None,
                false => Some(validator_flags(cfg)?),
            };
            validator_handle = Some(start_test_validator(cfg, flags, true)?);
        }

        let url = cluster_url(cfg);

        let node_options = format!(
            "{} {}",
            match std::env::var_os("NODE_OPTIONS") {
                Some(value) => value
                    .into_string()
                    .map_err(std::env::VarError::NotUnicode)?,
                None => "".to_owned(),
            },
            get_node_dns_option()?,
        );

        // Setup log reader.
        let log_streams = stream_logs(cfg, &url);

        // Run the tests.
        let test_result: Result<_> = {
            let cmd = cfg
                .scripts
                .get("test")
                .expect("Not able to find command for `test`")
                .clone();
            let mut args: Vec<&str> = cmd
                .split(' ')
                .chain(extra_args.iter().map(|arg| arg.as_str()))
                .collect();
            let program = args.remove(0);

            std::process::Command::new(program)
                .args(args)
                .env("ANCHOR_PROVIDER_URL", url)
                .env("ANCHOR_WALLET", cfg.provider.wallet.to_string())
                .env("NODE_OPTIONS", node_options)
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .output()
                .map_err(anyhow::Error::from)
                .context(cmd)
        };

        // Keep validator running if needed.
        if test_result.is_ok() && detach {
            println!("Local validator still running. Press Ctrl + C quit.");
            std::io::stdin().lock().lines().next().unwrap().unwrap();
        }

        // Check all errors and shut down.
        if let Some(mut child) = validator_handle {
            if let Err(err) = child.kill() {
                println!("Failed to kill subprocess {}: {}", child.id(), err);
            }
        }
        for mut child in log_streams? {
            if let Err(err) = child.kill() {
                println!("Failed to kill subprocess {}: {}", child.id(), err);
            }
        }

        // Must exist *after* shutting down the validator and log streams.
        match test_result {
            Ok(exit) => {
                if !exit.status.success() {
                    std::process::exit(exit.status.code().unwrap());
                }
            }
            Err(err) => {
                println!("Failed to run test: {:#}", err)
            }
        }

        Ok(())
    })
}

// Returns the solana-test-validator flags. This will embed the workspace
// programs in the genesis block so we don't have to deploy every time. It also
// allows control of other solana-test-validator features.
fn validator_flags(cfg: &WithPath<Config>) -> Result<Vec<String>> {
    let programs = cfg.programs.get(&Cluster::Localnet);

    let mut flags = Vec::new();
    for mut program in cfg.read_all_programs()? {
        let binary_path = program.binary_path().display().to_string();

        // Use the [programs.cluster] override and fallback to the keypair
        // files if no override is given.
        let address = programs
            .and_then(|m| m.get(&program.lib_name))
            .map(|deployment| Ok(deployment.address.to_string()))
            .unwrap_or_else(|| program.pubkey().map(|p| p.to_string()))?;

        flags.push("--bpf-program".to_string());
        flags.push(address.clone());
        flags.push(binary_path);

        if let Some(mut idl) = program.idl.as_mut() {
            // Add program address to the IDL.
            idl.metadata = Some(serde_json::to_value(IdlTestMetadata { address })?);

            // Persist it.
            let idl_out = PathBuf::from("target/idl")
                .join(&idl.name)
                .with_extension("json");
            write_idl(idl, OutFile::File(idl_out))?;
        }
    }

    if let Some(test) = cfg.test.as_ref() {
        if let Some(genesis) = &test.genesis {
            for entry in genesis {
                let program_path = Path::new(&entry.program);
                if !program_path.exists() {
                    return Err(anyhow!(
                        "Program in genesis configuration does not exist at path: {}",
                        program_path.display()
                    ));
                }
                flags.push("--bpf-program".to_string());
                flags.push(entry.address.clone());
                flags.push(entry.program.clone());
            }
        }
        if let Some(clone) = &test.clone {
            for entry in clone {
                flags.push("--clone".to_string());
                flags.push(entry.address.clone());
            }
        }
        if let Some(validator) = &test.validator {
            for (key, value) in serde_json::to_value(validator)?.as_object().unwrap() {
                if key == "ledger" {
                    continue;
                };
                flags.push(format!("--{}", key.replace("_", "-")));
                if let serde_json::Value::String(v) = value {
                    flags.push(v.to_string());
                } else {
                    flags.push(value.to_string());
                }
            }
        }
    }

    Ok(flags)
}

fn stream_logs(config: &WithPath<Config>, rpc_url: &str) -> Result<Vec<std::process::Child>> {
    let program_logs_dir = ".anchor/program-logs";
    if Path::new(program_logs_dir).exists() {
        fs::remove_dir_all(program_logs_dir)?;
    }
    fs::create_dir_all(program_logs_dir)?;
    let mut handles = vec![];
    for program in config.read_all_programs()? {
        let mut file = File::open(&format!("target/idl/{}.json", program.lib_name))?;
        let mut contents = vec![];
        file.read_to_end(&mut contents)?;
        let idl: Idl = serde_json::from_slice(&contents)?;
        let metadata = idl
            .metadata
            .ok_or_else(|| anyhow!("Program address not found."))?;
        let metadata: IdlTestMetadata = serde_json::from_value(metadata)?;

        let log_file = File::create(format!(
            "{}/{}.{}.log",
            program_logs_dir, metadata.address, program.lib_name,
        ))?;
        let stdio = std::process::Stdio::from(log_file);
        let child = std::process::Command::new("solana")
            .arg("logs")
            .arg(metadata.address)
            .arg("--url")
            .arg(rpc_url)
            .stdout(stdio)
            .spawn()?;
        handles.push(child);
    }
    if let Some(test) = config.test.as_ref() {
        if let Some(genesis) = &test.genesis {
            for entry in genesis {
                let log_file = File::create(format!("{}/{}.log", program_logs_dir, entry.address))?;
                let stdio = std::process::Stdio::from(log_file);
                let child = std::process::Command::new("solana")
                    .arg("logs")
                    .arg(entry.address.clone())
                    .arg("--url")
                    .arg(rpc_url)
                    .stdout(stdio)
                    .spawn()?;
                handles.push(child);
            }
        }
    }

    Ok(handles)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdlTestMetadata {
    address: String,
}

fn start_test_validator(
    cfg: &Config,
    flags: Option<Vec<String>>,
    test_log_stdout: bool,
) -> Result<Child> {
    //
    let (test_ledger_directory, test_ledger_log_filename) = test_validator_file_paths(cfg);

    // Start a validator for testing.
    let (test_validator_stdout, test_validator_stderr) = match test_log_stdout {
        true => {
            let test_validator_stdout_file = File::create(&test_ledger_log_filename)?;
            let test_validator_sterr_file = test_validator_stdout_file.try_clone()?;
            (
                Stdio::from(test_validator_stdout_file),
                Stdio::from(test_validator_sterr_file),
            )
        }
        false => (Stdio::inherit(), Stdio::inherit()),
    };

    let rpc_url = test_validator_rpc_url(cfg);

    let mut validator_handle = std::process::Command::new("solana-test-validator")
        .arg("--ledger")
        .arg(test_ledger_directory)
        .arg("--mint")
        .arg(cfg.wallet_kp()?.pubkey().to_string())
        .args(flags.unwrap_or_default())
        .stdout(test_validator_stdout)
        .stderr(test_validator_stderr)
        .spawn()
        .map_err(|e| anyhow::format_err!("{}", e.to_string()))?;

    // Wait for the validator to be ready.
    let client = RpcClient::new(rpc_url);
    let mut count = 0;
    let ms_wait = cfg
        .test
        .as_ref()
        .and_then(|test| test.startup_wait)
        .unwrap_or(5_000);
    while count < ms_wait {
        let r = client.get_recent_blockhash();
        if r.is_ok() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
        count += 1;
    }
    if count == ms_wait {
        eprintln!(
            "Unable to start test validator. Check {} for errors.",
            test_ledger_log_filename
        );
        validator_handle.kill()?;
        std::process::exit(1);
    }
    Ok(validator_handle)
}

// Return the URL that solana-test-validator should be running on given the
// configuration
fn test_validator_rpc_url(cfg: &Config) -> String {
    match &cfg.test.as_ref() {
        Some(Test {
            validator: Some(validator),
            ..
        }) => format!("http://{}:{}", validator.bind_address, validator.rpc_port),
        _ => "http://localhost:8899".to_string(),
    }
}

// Setup and return paths to the solana-test-validator ledger directory and log
// files given the configuration
fn test_validator_file_paths(cfg: &Config) -> (String, String) {
    let ledger_directory = match &cfg.test.as_ref() {
        Some(Test {
            validator: Some(validator),
            ..
        }) => &validator.ledger,
        _ => ".anchor/test-ledger",
    };

    if !Path::new(&ledger_directory).is_relative() {
        // Prevent absolute paths to avoid someone using / or similar, as the
        // directory gets removed
        eprintln!("Ledger directory {} must be relative", ledger_directory);
        std::process::exit(1);
    }
    if Path::new(&ledger_directory).exists() {
        fs::remove_dir_all(&ledger_directory).unwrap();
    }

    fs::create_dir_all(&ledger_directory).unwrap();

    (
        ledger_directory.to_string(),
        format!("{}/test-ledger-log.txt", ledger_directory),
    )
}

fn cluster_url(cfg: &Config) -> String {
    let is_localnet = cfg.provider.cluster == Cluster::Localnet;
    match is_localnet {
        // Cluster is Localnet, assume the intent is to use the configuration
        // for solana-test-validator
        true => test_validator_rpc_url(cfg),
        false => cfg.provider.cluster.url().to_string(),
    }
}

fn deploy(cfg_override: &ConfigOverride, program_str: Option<String>) -> Result<()> {
    with_workspace(cfg_override, |cfg| {
        let url = cluster_url(cfg);
        let keypair = cfg.provider.wallet.to_string();

        // Deploy the programs.
        println!("Deploying workspace: {}", url);
        println!("Upgrade authority: {}", keypair);

        for mut program in cfg.read_all_programs()? {
            if let Some(single_prog_str) = &program_str {
                let program_name = program.path.file_name().unwrap().to_str().unwrap();
                if single_prog_str.as_str() != program_name {
                    continue;
                }
            }
            let binary_path = program.binary_path().display().to_string();

            println!(
                "Deploying program {:?}...",
                program.path.file_name().unwrap().to_str().unwrap()
            );
            println!("Program path: {}...", binary_path);

            let file = program.keypair_file()?;

            // Send deploy transactions.
            let exit = std::process::Command::new("solana")
                .arg("program")
                .arg("deploy")
                .arg("--url")
                .arg(&url)
                .arg("--keypair")
                .arg(&keypair)
                .arg("--program-id")
                .arg(file.path().display().to_string())
                .arg(&binary_path)
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .output()
                .expect("Must deploy");
            if !exit.status.success() {
                println!("There was a problem deploying: {:?}.", exit);
                std::process::exit(exit.status.code().unwrap_or(1));
            }

            let program_pubkey = program.pubkey()?;
            if let Some(mut idl) = program.idl.as_mut() {
                // Add program address to the IDL.
                idl.metadata = Some(serde_json::to_value(IdlTestMetadata {
                    address: program_pubkey.to_string(),
                })?);

                // Persist it.
                let idl_out = PathBuf::from("target/idl")
                    .join(&idl.name)
                    .with_extension("json");
                write_idl(idl, OutFile::File(idl_out))?;
            }
        }

        println!("Deploy success");

        Ok(())
    })
}

fn upgrade(
    cfg_override: &ConfigOverride,
    program_id: Pubkey,
    program_filepath: String,
) -> Result<()> {
    let path: PathBuf = program_filepath.parse().unwrap();
    let program_filepath = path.canonicalize()?.display().to_string();

    with_workspace(cfg_override, |cfg| {
        let url = cluster_url(cfg);
        let exit = std::process::Command::new("solana")
            .arg("program")
            .arg("deploy")
            .arg("--url")
            .arg(url)
            .arg("--keypair")
            .arg(&cfg.provider.wallet.to_string())
            .arg("--program-id")
            .arg(program_id.to_string())
            .arg(&program_filepath)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .expect("Must deploy");
        if !exit.status.success() {
            println!("There was a problem deploying: {:?}.", exit);
            std::process::exit(exit.status.code().unwrap_or(1));
        }
        Ok(())
    })
}

fn create_idl_account(
    cfg: &Config,
    keypair_path: &str,
    program_id: &Pubkey,
    idl: &Idl,
) -> Result<Pubkey> {
    // Misc.
    let idl_address = IdlAccount::address(program_id);
    let keypair = solana_sdk::signature::read_keypair_file(keypair_path)
        .map_err(|_| anyhow!("Unable to read keypair file"))?;
    let url = cluster_url(cfg);
    let client = RpcClient::new(url);
    let idl_data = serialize_idl(idl)?;

    // Run `Create instruction.
    {
        let data = serialize_idl_ix(anchor_lang::idl::IdlInstruction::Create {
            data_len: (idl_data.len() as u64) * 2, // Double for future growth.
        })?;
        let program_signer = Pubkey::find_program_address(&[], program_id).0;
        let accounts = vec![
            AccountMeta::new_readonly(keypair.pubkey(), true),
            AccountMeta::new(idl_address, false),
            AccountMeta::new_readonly(program_signer, false),
            AccountMeta::new_readonly(solana_program::system_program::ID, false),
            AccountMeta::new_readonly(*program_id, false),
            AccountMeta::new_readonly(solana_program::sysvar::rent::ID, false),
        ];
        let ix = Instruction {
            program_id: *program_id,
            accounts,
            data,
        };
        let (recent_hash, _fee_calc) = client.get_recent_blockhash()?;
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&keypair.pubkey()),
            &[&keypair],
            recent_hash,
        );
        client.send_and_confirm_transaction_with_spinner_and_config(
            &tx,
            CommitmentConfig::confirmed(),
            RpcSendTransactionConfig {
                skip_preflight: true,
                ..RpcSendTransactionConfig::default()
            },
        )?;
    }

    // Write directly to the IDL account buffer.
    idl_write(cfg, program_id, idl, IdlAccount::address(program_id))?;

    Ok(idl_address)
}

fn create_idl_buffer(
    cfg: &Config,
    keypair_path: &str,
    program_id: &Pubkey,
    idl: &Idl,
) -> Result<Pubkey> {
    let keypair = solana_sdk::signature::read_keypair_file(keypair_path)
        .map_err(|_| anyhow!("Unable to read keypair file"))?;
    let url = cluster_url(cfg);
    let client = RpcClient::new(url);

    let buffer = Keypair::generate(&mut OsRng);

    // Creates the new buffer account with the system program.
    let create_account_ix = {
        let space = 8 + 32 + 4 + serialize_idl(idl)?.len() as usize;
        let lamports = client.get_minimum_balance_for_rent_exemption(space)?;
        solana_sdk::system_instruction::create_account(
            &keypair.pubkey(),
            &buffer.pubkey(),
            lamports,
            space as u64,
            program_id,
        )
    };

    // Program instruction to create the buffer.
    let create_buffer_ix = {
        let accounts = vec![
            AccountMeta::new(buffer.pubkey(), false),
            AccountMeta::new_readonly(keypair.pubkey(), true),
            AccountMeta::new_readonly(sysvar::rent::ID, false),
        ];
        let mut data = anchor_lang::idl::IDL_IX_TAG.to_le_bytes().to_vec();
        data.append(&mut IdlInstruction::CreateBuffer.try_to_vec()?);
        Instruction {
            program_id: *program_id,
            accounts,
            data,
        }
    };

    // Build the transaction.
    let (recent_hash, _fee_calc) = client.get_recent_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[create_account_ix, create_buffer_ix],
        Some(&keypair.pubkey()),
        &[&keypair, &buffer],
        recent_hash,
    );

    // Send the transaction.
    client.send_and_confirm_transaction_with_spinner_and_config(
        &tx,
        CommitmentConfig::confirmed(),
        RpcSendTransactionConfig {
            skip_preflight: true,
            ..RpcSendTransactionConfig::default()
        },
    )?;

    Ok(buffer.pubkey())
}

// Serialize and compress the idl.
fn serialize_idl(idl: &Idl) -> Result<Vec<u8>> {
    let json_bytes = serde_json::to_vec(idl)?;
    let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
    e.write_all(&json_bytes)?;
    e.finish().map_err(Into::into)
}

fn serialize_idl_ix(ix_inner: anchor_lang::idl::IdlInstruction) -> Result<Vec<u8>> {
    let mut data = anchor_lang::idl::IDL_IX_TAG.to_le_bytes().to_vec();
    data.append(&mut ix_inner.try_to_vec()?);
    Ok(data)
}

fn migrate(cfg_override: &ConfigOverride) -> Result<()> {
    with_workspace(cfg_override, |cfg| {
        println!("Running migration deploy script");

        let url = cluster_url(cfg);
        let cur_dir = std::env::current_dir()?;

        let use_ts =
            Path::new("tsconfig.json").exists() && Path::new("migrations/deploy.ts").exists();

        if !Path::new(".anchor").exists() {
            fs::create_dir(".anchor")?;
        }
        std::env::set_current_dir(".anchor")?;

        let exit = if use_ts {
            let module_path = cur_dir.join("migrations/deploy.ts");
            let deploy_script_host_str =
                template::deploy_ts_script_host(&url, &module_path.display().to_string());
            fs::write("deploy.ts", deploy_script_host_str)?;
            std::process::Command::new("ts-node")
                .arg("deploy.ts")
                .env("ANCHOR_WALLET", cfg.provider.wallet.to_string())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .output()?
        } else {
            let module_path = cur_dir.join("migrations/deploy.js");
            let deploy_script_host_str =
                template::deploy_js_script_host(&url, &module_path.display().to_string());
            fs::write("deploy.js", deploy_script_host_str)?;
            std::process::Command::new("node")
                .arg("deploy.js")
                .env("ANCHOR_WALLET", cfg.provider.wallet.to_string())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .output()?
        };

        if !exit.status.success() {
            println!("Deploy failed.");
            std::process::exit(exit.status.code().unwrap());
        }

        println!("Deploy complete.");
        Ok(())
    })
}

fn set_workspace_dir_or_exit() {
    let d = match Config::discover(&ConfigOverride::default()) {
        Err(err) => {
            println!("Workspace configuration error: {}", err);
            std::process::exit(1);
        }
        Ok(d) => d,
    };
    match d {
        None => {
            println!("Not in anchor workspace.");
            std::process::exit(1);
        }
        Some(cfg) => {
            match cfg.path().parent() {
                None => {
                    println!("Unable to make new program");
                }
                Some(parent) => {
                    if std::env::set_current_dir(&parent).is_err() {
                        println!("Not in anchor workspace.");
                        std::process::exit(1);
                    }
                }
            };
        }
    }
}

#[cfg(feature = "dev")]
fn airdrop(cfg_override: &ConfigOverride) -> Result<()> {
    let url = cfg_override
        .cluster
        .unwrap_or_else(|| "https://api.devnet.solana.com".to_string());
    loop {
        let exit = std::process::Command::new("solana")
            .arg("airdrop")
            .arg("10")
            .arg("--url")
            .arg(&url)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .expect("Must airdrop");
        if !exit.status.success() {
            println!("There was a problem airdropping: {:?}.", exit);
            std::process::exit(exit.status.code().unwrap_or(1));
        }
        std::thread::sleep(std::time::Duration::from_millis(10000));
    }
}

fn cluster(_cmd: ClusterCommand) -> Result<()> {
    println!("Cluster Endpoints:\n");
    println!("* Mainnet - https://solana-api.projectserum.com");
    println!("* Mainnet - https://api.mainnet-beta.solana.com");
    println!("* Devnet  - https://api.devnet.solana.com");
    println!("* Testnet - https://api.testnet.solana.com");
    Ok(())
}

fn shell(cfg_override: &ConfigOverride) -> Result<()> {
    with_workspace(cfg_override, |cfg| {
        let programs = {
            // Create idl map from all workspace programs.
            let mut idls: HashMap<String, Idl> = cfg
                .read_all_programs()?
                .iter()
                .filter(|program| program.idl.is_some())
                .map(|program| {
                    (
                        program.idl.as_ref().unwrap().name.clone(),
                        program.idl.clone().unwrap(),
                    )
                })
                .collect();
            // Insert all manually specified idls into the idl map.
            if let Some(programs) = cfg.programs.get(&cfg.provider.cluster) {
                let _ = programs
                    .iter()
                    .map(|(name, pd)| {
                        if let Some(idl_fp) = &pd.idl {
                            let file_str =
                                fs::read_to_string(idl_fp).expect("Unable to read IDL file");
                            let idl = serde_json::from_str(&file_str).expect("Idl not readable");
                            idls.insert(name.clone(), idl);
                        }
                    })
                    .collect::<Vec<_>>();
            }

            // Finalize program list with all programs with IDLs.
            match cfg.programs.get(&cfg.provider.cluster) {
                None => Vec::new(),
                Some(programs) => programs
                    .iter()
                    .filter_map(|(name, program_deployment)| {
                        Some(ProgramWorkspace {
                            name: name.to_string(),
                            program_id: program_deployment.address,
                            idl: match idls.get(name) {
                                None => return None,
                                Some(idl) => idl.clone(),
                            },
                        })
                    })
                    .collect::<Vec<ProgramWorkspace>>(),
            }
        };
        let url = cluster_url(cfg);
        let js_code = template::node_shell(&url, &cfg.provider.wallet.to_string(), programs)?;
        let mut child = std::process::Command::new("node")
            .args(&["-e", &js_code, "-i", "--experimental-repl-await"])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| anyhow::format_err!("{}", e.to_string()))?;

        if !child.wait()?.success() {
            println!("Error running node shell");
            return Ok(());
        }
        Ok(())
    })
}

fn run(cfg_override: &ConfigOverride, script: String) -> Result<()> {
    with_workspace(cfg_override, |cfg| {
        let url = cluster_url(cfg);
        let script = cfg
            .scripts
            .get(&script)
            .ok_or_else(|| anyhow!("Unable to find script"))?;
        let exit = std::process::Command::new("bash")
            .arg("-c")
            .arg(&script)
            .env("ANCHOR_PROVIDER_URL", url)
            .env("ANCHOR_WALLET", cfg.provider.wallet.to_string())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .unwrap();
        if !exit.status.success() {
            std::process::exit(exit.status.code().unwrap_or(1));
        }
        Ok(())
    })
}

fn login(_cfg_override: &ConfigOverride, token: String) -> Result<()> {
    let dir = shellexpand::tilde("~/.config/anchor");
    if !Path::new(&dir.to_string()).exists() {
        fs::create_dir(dir.to_string())?;
    }

    std::env::set_current_dir(dir.to_string())?;

    // Freely overwrite the entire file since it's not used for anything else.
    let mut file = File::create("credentials")?;
    file.write_all(template::credentials(&token).as_bytes())?;
    Ok(())
}

fn publish(
    cfg_override: &ConfigOverride,
    program_name: String,
    cargo_args: Vec<String>,
) -> Result<()> {
    // Discover the various workspace configs.
    let cfg = Config::discover(cfg_override)?.expect("Not in workspace.");

    let program = cfg
        .get_program(&program_name)?
        .ok_or_else(|| anyhow!("Workspace member not found"))?;

    let program_cargo_lock = pathdiff::diff_paths(
        program.path().join("Cargo.lock"),
        cfg.path().parent().unwrap(),
    )
    .ok_or_else(|| anyhow!("Unable to diff Cargo.lock path"))?;
    let cargo_lock = Path::new("Cargo.lock");

    // There must be a Cargo.lock
    if !program_cargo_lock.exists() && !cargo_lock.exists() {
        return Err(anyhow!("Cargo.lock must exist for a verifiable build"));
    }

    println!("Publishing will make your code public. Are you sure? Enter (yes)/no:");

    let answer = std::io::stdin().lock().lines().next().unwrap().unwrap();
    if answer != "yes" {
        println!("Aborting");
        return Ok(());
    }

    let anchor_package = AnchorPackage::from(program_name.clone(), &cfg)?;
    let anchor_package_bytes = serde_json::to_vec(&anchor_package)?;

    // Set directory to top of the workspace.
    let workspace_dir = cfg.path().parent().unwrap();
    std::env::set_current_dir(workspace_dir)?;

    // Create the workspace tarball.
    let dot_anchor = workspace_dir.join(".anchor");
    fs::create_dir_all(&dot_anchor)?;
    let tarball_filename = dot_anchor.join(format!("{}.tar.gz", program_name));
    let tar_gz = File::create(&tarball_filename)?;
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = tar::Builder::new(enc);

    // Files that will always be included if they exist.
    println!("PACKING: Anchor.toml");
    tar.append_path("Anchor.toml")?;
    if cargo_lock.exists() {
        println!("PACKING: Cargo.lock");
        tar.append_path(cargo_lock)?;
    }
    if Path::new("Cargo.toml").exists() {
        println!("PACKING: Cargo.toml");
        tar.append_path("Cargo.toml")?;
    }
    if Path::new("LICENSE").exists() {
        println!("PACKING: LICENSE");
        tar.append_path("LICENSE")?;
    }
    if Path::new("README.md").exists() {
        println!("PACKING: README.md");
        tar.append_path("README.md")?;
    }

    // All workspace programs.
    for path in cfg.get_program_list()? {
        let mut dirs = walkdir::WalkDir::new(&path)
            .into_iter()
            .filter_entry(|e| !is_hidden(e));

        // Skip the parent dir.
        let _ = dirs.next().unwrap()?;

        for entry in dirs {
            let e = entry.map_err(|e| anyhow!("{:?}", e))?;

            let e = pathdiff::diff_paths(e.path(), cfg.path().parent().unwrap())
                .ok_or_else(|| anyhow!("Unable to diff paths"))?;

            let path_str = e.display().to_string();

            // Skip target dir.
            if !path_str.contains("target/") && !path_str.contains("/target") {
                // Only add the file if it's not empty.
                let metadata = fs::File::open(&e)?.metadata()?;
                if metadata.len() > 0 {
                    println!("PACKING: {}", e.display());
                    if e.is_dir() {
                        tar.append_dir_all(&e, &e)?;
                    } else {
                        tar.append_path(&e)?;
                    }
                }
            }
        }
    }

    // Tar pack complete.
    tar.into_inner()?;

    // Create tmp directory for workspace.
    let ws_dir = dot_anchor.join("workspace");
    if Path::exists(&ws_dir) {
        fs::remove_dir_all(&ws_dir)?;
    }
    fs::create_dir_all(&ws_dir)?;

    // Unpack the archive into the new workspace directory.
    std::env::set_current_dir(&ws_dir)?;
    unpack_archive(&tarball_filename)?;

    // Build the program before sending it to the server.
    build(
        cfg_override,
        None,
        None,
        true,
        Some(program_name),
        cfg.solana_version.clone(),
        None,
        None,
        cargo_args,
    )?;

    // Success. Now we can finally upload to the server without worrying
    // about a build failure.

    // Upload the tarball to the server.
    let token = registry_api_token(cfg_override)?;
    let form = Form::new()
        .part("manifest", Part::bytes(anchor_package_bytes))
        .part("workspace", {
            let file = File::open(&tarball_filename)?;
            Part::reader(file)
        });
    let client = Client::new();
    let resp = client
        .post(&format!("{}/api/v0/build", cfg.registry.url))
        .bearer_auth(token)
        .multipart(form)
        .send()?;

    if resp.status() == 200 {
        println!("Build triggered");
    } else {
        println!(
            "{:?}",
            resp.text().unwrap_or_else(|_| "Server error".to_string())
        );
    }

    Ok(())
}

// Unpacks the tarball into the current directory.
fn unpack_archive(tar_path: impl AsRef<Path>) -> Result<()> {
    let tar = GzDecoder::new(std::fs::File::open(tar_path)?);
    let mut archive = Archive::new(tar);
    archive.unpack(".")?;
    archive.into_inner();

    Ok(())
}

fn registry_api_token(_cfg_override: &ConfigOverride) -> Result<String> {
    #[derive(Debug, Deserialize)]
    struct Registry {
        token: String,
    }
    #[derive(Debug, Deserialize)]
    struct Credentials {
        registry: Registry,
    }
    let filename = shellexpand::tilde("~/.config/anchor/credentials");
    let mut file = File::open(filename.to_string())?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let credentials_toml: Credentials = toml::from_str(&contents)?;

    Ok(credentials_toml.registry.token)
}

fn keys(cfg_override: &ConfigOverride, cmd: KeysCommand) -> Result<()> {
    match cmd {
        KeysCommand::List => keys_list(cfg_override),
    }
}

fn keys_list(cfg_override: &ConfigOverride) -> Result<()> {
    let cfg = Config::discover(cfg_override)?.expect("Not in workspace.");
    for program in cfg.read_all_programs()? {
        let pubkey = program.pubkey()?;
        println!("{}: {}", program.lib_name, pubkey);
    }
    Ok(())
}

fn localnet(
    cfg_override: &ConfigOverride,
    skip_build: bool,
    skip_deploy: bool,
    cargo_args: Vec<String>,
) -> Result<()> {
    with_workspace(cfg_override, |cfg| {
        // Build if needed.
        if !skip_build {
            build(
                cfg_override,
                None,
                None,
                false,
                None,
                None,
                None,
                None,
                cargo_args,
            )?;
        }

        let flags = match skip_deploy {
            true => None,
            false => Some(validator_flags(cfg)?),
        };

        let validator_handle = &mut start_test_validator(cfg, flags, false)?;

        // Setup log reader.
        let url = test_validator_rpc_url(cfg);
        let log_streams = stream_logs(cfg, &url);

        std::io::stdin().lock().lines().next().unwrap().unwrap();

        // Check all errors and shut down.
        if let Err(err) = validator_handle.kill() {
            println!(
                "Failed to kill subprocess {}: {}",
                validator_handle.id(),
                err
            );
        }

        for mut child in log_streams? {
            if let Err(err) = child.kill() {
                println!("Failed to kill subprocess {}: {}", child.id(), err);
            }
        }

        Ok(())
    })
}

// with_workspace ensures the current working directory is always the top level
// workspace directory, i.e., where the `Anchor.toml` file is located, before
// and after the closure invocation.
//
// The closure passed into this function must never change the working directory
// to be outside the workspace. Doing so will have undefined behavior.
fn with_workspace<R>(cfg_override: &ConfigOverride, f: impl FnOnce(&WithPath<Config>) -> R) -> R {
    set_workspace_dir_or_exit();

    let cfg = Config::discover(cfg_override)
        .expect("Previously set the workspace dir")
        .expect("Anchor.toml must always exist");

    let r = f(&cfg);

    set_workspace_dir_or_exit();

    r
}

fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s == "." || s.starts_with('.') || s == "target")
        .unwrap_or(false)
}

fn get_node_version() -> Result<Version> {
    let node_version = std::process::Command::new("node")
        .arg("--version")
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| anyhow::format_err!("node failed: {}", e.to_string()))?;
    let output = std::str::from_utf8(&node_version.stdout)?
        .strip_prefix('v')
        .unwrap()
        .trim();
    Version::parse(output).map_err(Into::into)
}

fn get_node_dns_option() -> Result<&'static str> {
    let version = get_node_version()?;
    let req = VersionReq::parse(">=16.4.0").unwrap();
    let option = match req.matches(&version) {
        true => "--dns-result-order=ipv4first",
        false => "",
    };
    Ok(option)
}
