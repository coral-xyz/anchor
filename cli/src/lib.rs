use crate::config::{
    AnchorPackage, BootstrapMode, BuildConfig, Config, ConfigOverride, Manifest, ProgramArch,
    ProgramDeployment, ProgramWorkspace, ScriptsConfig, TestValidator, WithPath, SHUTDOWN_WAIT,
    STARTUP_WAIT,
};
use anchor_client::Cluster;
use anchor_lang::idl::{IdlAccount, IdlInstruction, ERASED_AUTHORITY};
use anchor_lang::{AccountDeserialize, AnchorDeserialize, AnchorSerialize};
use anchor_syn::idl::{EnumFields, Idl, IdlType, IdlTypeDefinitionTy};
use anyhow::{anyhow, Context, Result};
use clap::Parser;
use flate2::read::GzDecoder;
use flate2::read::ZlibDecoder;
use flate2::write::{GzEncoder, ZlibEncoder};
use flate2::Compression;
use heck::{ToKebabCase, ToSnakeCase};
use regex::RegexBuilder;
use reqwest::blocking::multipart::{Form, Part};
use reqwest::blocking::Client;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value as JsonValue};
use solana_client::rpc_client::RpcClient;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_sdk::account_utils::StateMut;
use solana_sdk::bpf_loader;
use solana_sdk::bpf_loader_deprecated;
use solana_sdk::bpf_loader_upgradeable::{self, UpgradeableLoaderState};
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signature::Signer;
use solana_sdk::sysvar;
use solana_sdk::transaction::Transaction;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};
use std::str::FromStr;
use std::string::ToString;
use tar::Archive;

pub mod config;
mod path;
pub mod rust_template;
pub mod solidity_template;

// Version of the docker image.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DOCKER_BUILDER_VERSION: &str = VERSION;

#[derive(Debug, Parser)]
#[clap(version = VERSION)]
pub struct Opts {
    #[clap(flatten)]
    pub cfg_override: ConfigOverride,
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Parser)]
pub enum Command {
    /// Initializes a workspace.
    Init {
        name: String,
        #[clap(short, long)]
        javascript: bool,
        #[clap(short, long)]
        solidity: bool,
        #[clap(long)]
        no_git: bool,
        #[clap(long)]
        jest: bool,
    },
    /// Builds the workspace.
    #[clap(name = "build", alias = "b")]
    Build {
        /// Output directory for the IDL.
        #[clap(short, long)]
        idl: Option<String>,
        /// True if the build should not fail even if there are
        /// no "CHECK" comments where normally required
        #[clap(long)]
        skip_lint: bool,
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
        /// Docker image to use. For --verifiable builds only.
        #[clap(short, long)]
        docker_image: Option<String>,
        /// Bootstrap docker image from scratch, installing all requirements for
        /// verifiable builds. Only works for debian-based images.
        #[clap(value_enum, short, long, default_value = "none")]
        bootstrap: BootstrapMode,
        /// Environment variables to pass into the docker container
        #[clap(short, long, required = false)]
        env: Vec<String>,
        /// Arguments to pass to the underlying `cargo build-bpf` command
        #[clap(required = false, last = true)]
        cargo_args: Vec<String>,
        /// Suppress doc strings in IDL output
        #[clap(long)]
        no_docs: bool,
        /// Architecture to use when building the program
        #[clap(value_enum, long, default_value = "bpf")]
        arch: ProgramArch,
    },
    /// Expands macros (wrapper around cargo expand)
    ///
    /// Use it in a program folder to expand program
    ///
    /// Use it in a workspace but outside a program
    /// folder to expand the entire workspace
    Expand {
        /// Expand only this program
        #[clap(short, long)]
        program_name: Option<String>,
        /// Arguments to pass to the underlying `cargo expand` command
        #[clap(required = false, last = true)]
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
        /// Docker image to use. For --verifiable builds only.
        #[clap(short, long)]
        docker_image: Option<String>,
        /// Bootstrap docker image from scratch, installing all requirements for
        /// verifiable builds. Only works for debian-based images.
        #[clap(value_enum, short, long, default_value = "none")]
        bootstrap: BootstrapMode,
        /// Architecture to use when building the program
        #[clap(value_enum, long, default_value = "bpf")]
        arch: ProgramArch,
        /// Environment variables to pass into the docker container
        #[clap(short, long, required = false)]
        env: Vec<String>,
        /// Arguments to pass to the underlying `cargo build-bpf` command.
        #[clap(required = false, last = true)]
        cargo_args: Vec<String>,
        /// Flag to skip building the program in the workspace,
        /// use this to save time when running verify and the program code is already built.
        #[clap(long, required = false)]
        skip_build: bool,
    },
    #[clap(name = "test", alias = "t")]
    /// Runs integration tests against a localnetwork.
    Test {
        /// Use this flag if you want to run tests against previously deployed
        /// programs.
        #[clap(long)]
        skip_deploy: bool,
        /// True if the build should not fail even if there are
        /// no "CHECK" comments where normally required
        #[clap(long)]
        skip_lint: bool,
        /// Flag to skip starting a local validator, if the configured cluster
        /// url is a localnet.
        #[clap(long)]
        skip_local_validator: bool,
        /// Flag to skip building the program in the workspace,
        /// use this to save time when running test and the program code is not altered.
        #[clap(long)]
        skip_build: bool,
        /// Architecture to use when building the program
        #[clap(value_enum, long, default_value = "bpf")]
        arch: ProgramArch,
        /// Flag to keep the local validator running after tests
        /// to be able to check the transactions.
        #[clap(long)]
        detach: bool,
        /// Run the test suites under the specified path
        #[clap(long)]
        run: Vec<String>,
        args: Vec<String>,
        /// Environment variables to pass into the docker container
        #[clap(short, long, required = false)]
        env: Vec<String>,
        /// Arguments to pass to the underlying `cargo build-bpf` command.
        #[clap(required = false, last = true)]
        cargo_args: Vec<String>,
    },
    /// Creates a new program.
    New {
        #[clap(short, long)]
        solidity: bool,
        name: String,
    },
    /// Commands for interacting with interface definitions.
    Idl {
        #[clap(subcommand)]
        subcmd: IdlCommand,
    },
    /// Remove all artifacts from the target directory except program keypairs.
    Clean,
    /// Deploys each program in the workspace.
    Deploy {
        /// Only deploy this program
        #[clap(short, long)]
        program_name: Option<String>,
        /// Keypair of the program (filepath) (requires program-name)
        #[clap(long, requires = "program_name")]
        program_keypair: Option<String>,
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
        /// Argument to pass to the underlying script.
        #[clap(required = false, last = true)]
        script_args: Vec<String>,
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
        /// Environment variables to pass into the docker container
        #[clap(short, long, required = false)]
        env: Vec<String>,
        /// Arguments to pass to the underlying `cargo build-bpf` command.
        #[clap(required = false, last = true)]
        cargo_args: Vec<String>,
        /// Flag to skip building the program in the workspace,
        /// use this to save time when publishing the program
        #[clap(long)]
        skip_build: bool,
        /// Architecture to use when building the program
        #[clap(value_enum, long, default_value = "bpf")]
        arch: ProgramArch,
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
        /// True if the build should not fail even if there are
        /// no "CHECK" comments where normally required
        #[clap(long)]
        skip_lint: bool,
        /// Architecture to use when building the program
        #[clap(value_enum, long, default_value = "bpf")]
        arch: ProgramArch,
        /// Environment variables to pass into the docker container
        #[clap(short, long, required = false)]
        env: Vec<String>,
        /// Arguments to pass to the underlying `cargo build-bpf` command.
        #[clap(required = false, last = true)]
        cargo_args: Vec<String>,
    },
    /// Fetch and deserialize an account using the IDL provided.
    Account {
        /// Account struct to deserialize
        account_type: String,
        /// Address of the account to deserialize
        address: Pubkey,
        /// IDL to use (defaults to workspace IDL)
        #[clap(long)]
        idl: Option<String>,
    },
}

#[derive(Debug, Parser)]
pub enum KeysCommand {
    /// List all of the program keys.
    List,
    /// Sync the program's `declare_id!` pubkey with the program's actual pubkey.
    Sync {
        /// Only sync the given program instead of all programs
        #[clap(short, long)]
        program_name: Option<String>,
    },
}

#[derive(Debug, Parser)]
pub enum IdlCommand {
    /// Initializes a program's IDL account. Can only be run once.
    Init {
        program_id: Pubkey,
        #[clap(short, long)]
        filepath: String,
    },
    Close {
        program_id: Pubkey,
        /// When used, the content of the instruction will only be printed in base64 form and not executed.
        /// Useful for multisig execution when the local wallet keypair is not available.
        #[clap(long)]
        print_only: bool,
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
        /// When used, the content of the instruction will only be printed in base64 form and not executed.
        /// Useful for multisig execution when the local wallet keypair is not available.
        #[clap(long)]
        print_only: bool,
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
        /// When used, the content of the instruction will only be printed in base64 form and not executed.
        /// Useful for multisig execution when the local wallet keypair is not available.
        #[clap(long)]
        print_only: bool,
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
        /// Suppress doc strings in output
        #[clap(long)]
        no_docs: bool,
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

#[derive(Debug, Parser)]
pub enum ClusterCommand {
    /// Prints common cluster urls.
    List,
}

pub fn entry(opts: Opts) -> Result<()> {
    match opts.command {
        Command::Init {
            name,
            javascript,
            solidity,
            no_git,
            jest,
        } => init(&opts.cfg_override, name, javascript, solidity, no_git, jest),
        Command::New { solidity, name } => new(&opts.cfg_override, solidity, name),
        Command::Build {
            idl,
            idl_ts,
            verifiable,
            program_name,
            solana_version,
            docker_image,
            bootstrap,
            cargo_args,
            env,
            skip_lint,
            no_docs,
            arch,
        } => build(
            &opts.cfg_override,
            idl,
            idl_ts,
            verifiable,
            skip_lint,
            program_name,
            solana_version,
            docker_image,
            bootstrap,
            None,
            None,
            env,
            cargo_args,
            no_docs,
            arch,
        ),
        Command::Verify {
            program_id,
            program_name,
            solana_version,
            docker_image,
            bootstrap,
            env,
            cargo_args,
            skip_build,
            arch,
        } => verify(
            &opts.cfg_override,
            program_id,
            program_name,
            solana_version,
            docker_image,
            bootstrap,
            env,
            cargo_args,
            skip_build,
            arch,
        ),
        Command::Clean => clean(&opts.cfg_override),
        Command::Deploy {
            program_name,
            program_keypair,
        } => deploy(&opts.cfg_override, program_name, program_keypair),
        Command::Expand {
            program_name,
            cargo_args,
        } => expand(&opts.cfg_override, program_name, &cargo_args),
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
            run,
            args,
            env,
            cargo_args,
            skip_lint,
            arch,
        } => test(
            &opts.cfg_override,
            skip_deploy,
            skip_local_validator,
            skip_build,
            skip_lint,
            detach,
            run,
            args,
            env,
            cargo_args,
            arch,
        ),
        #[cfg(feature = "dev")]
        Command::Airdrop { .. } => airdrop(&opts.cfg_override),
        Command::Cluster { subcmd } => cluster(subcmd),
        Command::Shell => shell(&opts.cfg_override),
        Command::Run {
            script,
            script_args,
        } => run(&opts.cfg_override, script, script_args),
        Command::Login { token } => login(&opts.cfg_override, token),
        Command::Publish {
            program,
            env,
            cargo_args,
            skip_build,
            arch,
        } => publish(
            &opts.cfg_override,
            program,
            env,
            cargo_args,
            skip_build,
            arch,
        ),
        Command::Keys { subcmd } => keys(&opts.cfg_override, subcmd),
        Command::Localnet {
            skip_build,
            skip_deploy,
            skip_lint,
            env,
            cargo_args,
            arch,
        } => localnet(
            &opts.cfg_override,
            skip_build,
            skip_deploy,
            skip_lint,
            env,
            cargo_args,
            arch,
        ),
        Command::Account {
            account_type,
            address,
            idl,
        } => account(&opts.cfg_override, account_type, address, idl),
    }
}

fn init(
    cfg_override: &ConfigOverride,
    name: String,
    javascript: bool,
    solidity: bool,
    no_git: bool,
    jest: bool,
) -> Result<()> {
    if Config::discover(cfg_override)?.is_some() {
        return Err(anyhow!("Workspace already initialized"));
    }

    // We need to format different cases for the dir and the name
    let rust_name = name.to_snake_case();
    let project_name = if name == rust_name {
        rust_name.clone()
    } else {
        name.to_kebab_case()
    };

    // Additional keywords that have not been added to the `syn` crate as reserved words
    // https://github.com/dtolnay/syn/pull/1098
    let extra_keywords = ["async", "await", "try"];
    // Anchor converts to snake case before writing the program name
    if syn::parse_str::<syn::Ident>(&rust_name).is_err()
        || extra_keywords.contains(&rust_name.as_str())
    {
        return Err(anyhow!(
            "Anchor workspace name must be a valid Rust identifier. It may not be a Rust reserved word, start with a digit, or include certain disallowed characters. See https://doc.rust-lang.org/reference/identifiers.html for more detail.",
        ));
    }

    fs::create_dir(&project_name)?;
    std::env::set_current_dir(&project_name)?;
    fs::create_dir("app")?;

    let mut cfg = Config::default();
    if jest {
        cfg.scripts.insert(
            "test".to_owned(),
            if javascript {
                "yarn run jest"
            } else {
                "yarn run jest --preset ts-jest"
            }
            .to_owned(),
        );
    } else {
        cfg.scripts.insert(
            "test".to_owned(),
            if javascript {
                "yarn run mocha -t 1000000 tests/"
            } else {
                "yarn run ts-mocha -p ./tsconfig.json -t 1000000 tests/**/*.ts"
            }
            .to_owned(),
        );
    }

    let mut localnet = BTreeMap::new();
    let program_id = if solidity {
        solidity_template::default_program_id()
    } else {
        rust_template::get_or_create_program_id(&rust_name)
    };
    localnet.insert(
        rust_name,
        ProgramDeployment {
            address: program_id,
            path: None,
            idl: None,
        },
    );
    cfg.programs.insert(Cluster::Localnet, localnet);
    let toml = cfg.to_string();
    fs::write("Anchor.toml", toml)?;

    // Initialize .gitignore file
    fs::write(".gitignore", rust_template::git_ignore())?;

    // Initialize .prettierignore file
    fs::write(".prettierignore", rust_template::prettier_ignore())?;

    // Build the program.
    if solidity {
        fs::create_dir("solidity")?;

        new_solidity_program(&project_name)?;
    } else {
        // Build virtual manifest for rust programs
        fs::write("Cargo.toml", rust_template::virtual_manifest())?;

        fs::create_dir("programs")?;

        new_rust_program(&project_name)?;
    }
    // Build the test suite.
    fs::create_dir("tests")?;
    // Build the migrations directory.
    fs::create_dir("migrations")?;

    if javascript {
        // Build javascript config
        let mut package_json = File::create("package.json")?;
        package_json.write_all(rust_template::package_json(jest).as_bytes())?;

        if jest {
            let mut test = File::create(format!("tests/{}.test.js", &project_name))?;
            if solidity {
                test.write_all(solidity_template::jest(&project_name).as_bytes())?;
            } else {
                test.write_all(rust_template::jest(&project_name).as_bytes())?;
            }
        } else {
            let mut test = File::create(format!("tests/{}.js", &project_name))?;
            if solidity {
                test.write_all(solidity_template::mocha(&project_name).as_bytes())?;
            } else {
                test.write_all(rust_template::mocha(&project_name).as_bytes())?;
            }
        }

        let mut deploy = File::create("migrations/deploy.js")?;

        deploy.write_all(rust_template::deploy_script().as_bytes())?;
    } else {
        // Build typescript config
        let mut ts_config = File::create("tsconfig.json")?;
        ts_config.write_all(rust_template::ts_config(jest).as_bytes())?;

        let mut ts_package_json = File::create("package.json")?;
        ts_package_json.write_all(rust_template::ts_package_json(jest).as_bytes())?;

        let mut deploy = File::create("migrations/deploy.ts")?;
        deploy.write_all(rust_template::ts_deploy_script().as_bytes())?;

        let mut mocha = File::create(format!("tests/{}.ts", &project_name))?;
        if solidity {
            mocha.write_all(solidity_template::ts_mocha(&project_name).as_bytes())?;
        } else {
            mocha.write_all(rust_template::ts_mocha(&project_name).as_bytes())?;
        }
    }

    let yarn_result = install_node_modules("yarn")?;
    if !yarn_result.status.success() {
        println!("Failed yarn install will attempt to npm install");
        install_node_modules("npm")?;
    }

    if !no_git {
        let git_result = std::process::Command::new("git")
            .arg("init")
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .map_err(|e| anyhow::format_err!("git init failed: {}", e.to_string()))?;
        if !git_result.status.success() {
            eprintln!("Failed to automatically initialize a new git repository");
        }
    }

    println!("{project_name} initialized");

    Ok(())
}

fn install_node_modules(cmd: &str) -> Result<std::process::Output> {
    if cfg!(target_os = "windows") {
        std::process::Command::new("cmd")
            .arg(format!("/C {cmd} install"))
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .map_err(|e| anyhow::format_err!("{} install failed: {}", cmd, e.to_string()))
    } else {
        std::process::Command::new(cmd)
            .arg("install")
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .map_err(|e| anyhow::format_err!("{} install failed: {}", cmd, e.to_string()))
    }
}

// Creates a new program crate in the `programs/<name>` directory.
fn new(cfg_override: &ConfigOverride, solidity: bool, name: String) -> Result<()> {
    with_workspace(cfg_override, |cfg| {
        match cfg.path().parent() {
            None => {
                println!("Unable to make new program");
            }
            Some(parent) => {
                std::env::set_current_dir(parent)?;

                let cluster = cfg.provider.cluster.clone();
                let programs = cfg.programs.entry(cluster).or_insert(BTreeMap::new());
                if programs.contains_key(&name) {
                    return Err(anyhow!("Program already exists"));
                }

                programs.insert(
                    name.clone(),
                    ProgramDeployment {
                        address: if solidity {
                            new_solidity_program(&name)?;
                            solidity_template::default_program_id()
                        } else {
                            new_rust_program(&name)?;
                            rust_template::get_or_create_program_id(&name)
                        },
                        path: None,
                        idl: None,
                    },
                );

                let toml = cfg.to_string();
                fs::write("Anchor.toml", toml)?;

                println!("Created new program.");
            }
        };
        Ok(())
    })
}

// Creates a new rust program crate in the current directory with `name`.
fn new_rust_program(name: &str) -> Result<()> {
    if !PathBuf::from("Cargo.toml").exists() {
        fs::write("Cargo.toml", rust_template::virtual_manifest())?;
    }
    fs::create_dir_all(format!("programs/{name}/src/"))?;
    let mut cargo_toml = File::create(format!("programs/{name}/Cargo.toml"))?;
    cargo_toml.write_all(rust_template::cargo_toml(name).as_bytes())?;
    let mut xargo_toml = File::create(format!("programs/{name}/Xargo.toml"))?;
    xargo_toml.write_all(rust_template::xargo_toml().as_bytes())?;
    let mut lib_rs = File::create(format!("programs/{name}/src/lib.rs"))?;
    lib_rs.write_all(rust_template::lib_rs(name).as_bytes())?;
    Ok(())
}

// Creates a new solidity program in the current directory with `name`.
fn new_solidity_program(name: &str) -> Result<()> {
    fs::create_dir_all("solidity")?;
    let mut lib_rs = File::create(format!("solidity/{name}.sol"))?;
    lib_rs.write_all(solidity_template::solidity(name).as_bytes())?;
    Ok(())
}

pub fn expand(
    cfg_override: &ConfigOverride,
    program_name: Option<String>,
    cargo_args: &[String],
) -> Result<()> {
    // Change to the workspace member directory, if needed.
    if let Some(program_name) = program_name.as_ref() {
        cd_member(cfg_override, program_name)?;
    }

    let workspace_cfg = Config::discover(cfg_override)?.expect("Not in workspace.");
    let cfg_parent = workspace_cfg.path().parent().expect("Invalid Anchor.toml");
    let cargo = Manifest::discover()?;

    let expansions_path = cfg_parent.join(".anchor/expanded-macros");
    fs::create_dir_all(&expansions_path)?;

    match cargo {
        // No Cargo.toml found, expand entire workspace
        None => expand_all(&workspace_cfg, expansions_path, cargo_args),
        // Cargo.toml is at root of workspace, expand entire workspace
        Some(cargo) if cargo.path().parent() == workspace_cfg.path().parent() => {
            expand_all(&workspace_cfg, expansions_path, cargo_args)
        }
        // Reaching this arm means Cargo.toml belongs to a single package. Expand it.
        Some(cargo) => expand_program(
            // If we found Cargo.toml, it must be in a directory so unwrap is safe
            cargo.path().parent().unwrap().to_path_buf(),
            expansions_path,
            cargo_args,
        ),
    }
}

fn expand_all(
    workspace_cfg: &WithPath<Config>,
    expansions_path: PathBuf,
    cargo_args: &[String],
) -> Result<()> {
    let cur_dir = std::env::current_dir()?;
    for p in workspace_cfg.get_rust_program_list()? {
        expand_program(p, expansions_path.clone(), cargo_args)?;
    }
    std::env::set_current_dir(cur_dir)?;
    Ok(())
}

fn expand_program(
    program_path: PathBuf,
    expansions_path: PathBuf,
    cargo_args: &[String],
) -> Result<()> {
    let cargo = Manifest::from_path(program_path.join("Cargo.toml"))
        .map_err(|_| anyhow!("Could not find Cargo.toml for program"))?;

    let target_dir_arg = {
        let mut target_dir_arg = OsString::from("--target-dir=");
        target_dir_arg.push(expansions_path.join("expand-target"));
        target_dir_arg
    };

    let package_name = &cargo
        .package
        .as_ref()
        .ok_or_else(|| anyhow!("Cargo config is missing a package"))?
        .name;
    let program_expansions_path = expansions_path.join(package_name);
    fs::create_dir_all(&program_expansions_path)?;

    let exit = std::process::Command::new("cargo")
        .arg("expand")
        .arg(target_dir_arg)
        .arg(&format!("--package={package_name}"))
        .args(cargo_args)
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| anyhow::format_err!("{}", e.to_string()))?;
    if !exit.status.success() {
        eprintln!("'anchor expand' failed. Perhaps you have not installed 'cargo-expand'? https://github.com/dtolnay/cargo-expand#installation");
        std::process::exit(exit.status.code().unwrap_or(1));
    }

    let version = cargo.version();
    let time = chrono::Utc::now().to_string().replace(' ', "_");
    let file_path = program_expansions_path.join(format!("{package_name}-{version}-{time}.rs"));
    fs::write(&file_path, &exit.stdout).map_err(|e| anyhow::format_err!("{}", e.to_string()))?;

    println!(
        "Expanded {} into file {}\n",
        package_name,
        file_path.to_string_lossy()
    );
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn build(
    cfg_override: &ConfigOverride,
    idl: Option<String>,
    idl_ts: Option<String>,
    verifiable: bool,
    skip_lint: bool,
    program_name: Option<String>,
    solana_version: Option<String>,
    docker_image: Option<String>,
    bootstrap: BootstrapMode,
    stdout: Option<File>, // Used for the package registry server.
    stderr: Option<File>, // Used for the package registry server.
    env_vars: Vec<String>,
    cargo_args: Vec<String>,
    no_docs: bool,
    arch: ProgramArch,
) -> Result<()> {
    // Change to the workspace member directory, if needed.
    if let Some(program_name) = program_name.as_ref() {
        cd_member(cfg_override, program_name)?;
    }

    let cfg = Config::discover(cfg_override)?.expect("Not in workspace.");
    let build_config = BuildConfig {
        verifiable,
        solana_version: solana_version.or_else(|| cfg.solana_version.clone()),
        docker_image: docker_image.unwrap_or_else(|| cfg.docker()),
        bootstrap,
    };
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

    if !&cfg.workspace.types.is_empty() {
        fs::create_dir_all(cfg_parent.join(&cfg.workspace.types))?;
    };

    match cargo {
        // No Cargo.toml so build the entire workspace.
        None => build_all(
            &cfg,
            cfg.path(),
            idl_out,
            idl_ts_out,
            &build_config,
            stdout,
            stderr,
            env_vars,
            cargo_args,
            skip_lint,
            no_docs,
            arch,
        )?,
        // If the Cargo.toml is at the root, build the entire workspace.
        Some(cargo) if cargo.path().parent() == cfg.path().parent() => build_all(
            &cfg,
            cfg.path(),
            idl_out,
            idl_ts_out,
            &build_config,
            stdout,
            stderr,
            env_vars,
            cargo_args,
            skip_lint,
            no_docs,
            arch,
        )?,
        // Cargo.toml represents a single package. Build it.
        Some(cargo) => build_rust_cwd(
            &cfg,
            cargo.path().to_path_buf(),
            idl_out,
            idl_ts_out,
            &build_config,
            stdout,
            stderr,
            env_vars,
            cargo_args,
            skip_lint,
            no_docs,
            &arch,
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
    build_config: &BuildConfig,
    stdout: Option<File>, // Used for the package registry server.
    stderr: Option<File>, // Used for the package registry server.
    env_vars: Vec<String>,
    cargo_args: Vec<String>,
    skip_lint: bool,
    no_docs: bool,
    arch: ProgramArch,
) -> Result<()> {
    let cur_dir = std::env::current_dir()?;
    let r = match cfg_path.parent() {
        None => Err(anyhow!("Invalid Anchor.toml at {}", cfg_path.display())),
        Some(_parent) => {
            for p in cfg.get_rust_program_list()? {
                build_rust_cwd(
                    cfg,
                    p.join("Cargo.toml"),
                    idl_out.clone(),
                    idl_ts_out.clone(),
                    build_config,
                    stdout.as_ref().map(|f| f.try_clone()).transpose()?,
                    stderr.as_ref().map(|f| f.try_clone()).transpose()?,
                    env_vars.clone(),
                    cargo_args.clone(),
                    skip_lint,
                    no_docs,
                    &arch,
                )?;
            }
            for (name, path) in cfg.get_solidity_program_list()? {
                build_solidity_cwd(
                    cfg,
                    name,
                    path,
                    idl_out.clone(),
                    idl_ts_out.clone(),
                    build_config,
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
fn build_rust_cwd(
    cfg: &WithPath<Config>,
    cargo_toml: PathBuf,
    idl_out: Option<PathBuf>,
    idl_ts_out: Option<PathBuf>,
    build_config: &BuildConfig,
    stdout: Option<File>,
    stderr: Option<File>,
    env_vars: Vec<String>,
    cargo_args: Vec<String>,
    skip_lint: bool,
    no_docs: bool,
    arch: &ProgramArch,
) -> Result<()> {
    match cargo_toml.parent() {
        None => return Err(anyhow!("Unable to find parent")),
        Some(p) => std::env::set_current_dir(p)?,
    };
    match build_config.verifiable {
        false => _build_rust_cwd(cfg, idl_out, idl_ts_out, skip_lint, arch, cargo_args),
        true => build_cwd_verifiable(
            cfg,
            cargo_toml,
            build_config,
            stdout,
            stderr,
            skip_lint,
            env_vars,
            cargo_args,
            no_docs,
            arch,
        ),
    }
}

// Runs the build command outside of a workspace.
#[allow(clippy::too_many_arguments)]
fn build_solidity_cwd(
    cfg: &WithPath<Config>,
    name: String,
    path: PathBuf,
    idl_out: Option<PathBuf>,
    idl_ts_out: Option<PathBuf>,
    build_config: &BuildConfig,
    stdout: Option<File>,
    stderr: Option<File>,
    cargo_args: Vec<String>,
) -> Result<()> {
    match path.parent() {
        None => return Err(anyhow!("Unable to find parent")),
        Some(p) => std::env::set_current_dir(p)?,
    };
    match build_config.verifiable {
        false => _build_solidity_cwd(
            cfg, &name, &path, idl_out, idl_ts_out, stdout, stderr, cargo_args,
        ),
        true => panic!("verifiable solidity not supported"),
    }
}

// Builds an anchor program in a docker image and copies the build artifacts
// into the `target/` directory.
#[allow(clippy::too_many_arguments)]
fn build_cwd_verifiable(
    cfg: &WithPath<Config>,
    cargo_toml: PathBuf,
    build_config: &BuildConfig,
    stdout: Option<File>,
    stderr: Option<File>,
    skip_lint: bool,
    env_vars: Vec<String>,
    cargo_args: Vec<String>,
    no_docs: bool,
    arch: &ProgramArch,
) -> Result<()> {
    // Create output dirs.
    let workspace_dir = cfg.path().parent().unwrap().canonicalize()?;
    fs::create_dir_all(workspace_dir.join("target/verifiable"))?;
    fs::create_dir_all(workspace_dir.join("target/idl"))?;
    fs::create_dir_all(workspace_dir.join("target/types"))?;
    if !&cfg.workspace.types.is_empty() {
        fs::create_dir_all(workspace_dir.join(&cfg.workspace.types))?;
    }

    let container_name = "anchor-program";

    // Build the binary in docker.
    let result = docker_build(
        cfg,
        container_name,
        cargo_toml,
        build_config,
        stdout,
        stderr,
        env_vars,
        cargo_args,
        arch,
    );

    match &result {
        Err(e) => {
            eprintln!("Error during Docker build: {e:?}");
        }
        Ok(_) => {
            // Build the idl.
            println!("Extracting the IDL");
            if let Ok(Some(idl)) = extract_idl(cfg, "src/lib.rs", skip_lint, no_docs) {
                // Write out the JSON file.
                println!("Writing the IDL file");
                let out_file = workspace_dir.join(format!("target/idl/{}.json", idl.name));
                write_idl(&idl, OutFile::File(out_file))?;

                // Write out the TypeScript type.
                println!("Writing the .ts file");
                let ts_file = workspace_dir.join(format!("target/types/{}.ts", idl.name));
                fs::write(&ts_file, rust_template::idl_ts(&idl)?)?;

                // Copy out the TypeScript type.
                if !&cfg.workspace.types.is_empty() {
                    fs::copy(
                        ts_file,
                        workspace_dir
                            .join(&cfg.workspace.types)
                            .join(idl.name)
                            .with_extension("ts"),
                    )?;
                }
            }
            println!("Build success");
        }
    }

    result
}

#[allow(clippy::too_many_arguments)]
fn docker_build(
    cfg: &WithPath<Config>,
    container_name: &str,
    cargo_toml: PathBuf,
    build_config: &BuildConfig,
    stdout: Option<File>,
    stderr: Option<File>,
    env_vars: Vec<String>,
    cargo_args: Vec<String>,
    arch: &ProgramArch,
) -> Result<()> {
    let binary_name = Manifest::from_path(&cargo_toml)?.lib_name()?;

    // Docker vars.
    let workdir = Path::new("/workdir");
    let volume_mount = format!(
        "{}:{}",
        cfg.path().parent().unwrap().canonicalize()?.display(),
        workdir.to_str().unwrap(),
    );
    println!("Using image {:?}", build_config.docker_image);

    // Start the docker image running detached in the background.
    let target_dir = workdir.join("docker-target");
    println!("Run docker image");
    let exit = std::process::Command::new("docker")
        .args([
            "run",
            "-it",
            "-d",
            "--name",
            container_name,
            "--env",
            &format!(
                "CARGO_TARGET_DIR={}",
                target_dir.as_path().to_str().unwrap()
            ),
            "-v",
            &volume_mount,
            "-w",
            workdir.to_str().unwrap(),
            &build_config.docker_image,
            "bash",
        ])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| anyhow::format_err!("Docker build failed: {}", e.to_string()))?;
    if !exit.status.success() {
        return Err(anyhow!("Failed to build program"));
    }

    let result = docker_prep(container_name, build_config).and_then(|_| {
        let cfg_parent = cfg.path().parent().unwrap();
        docker_build_bpf(
            container_name,
            cargo_toml.as_path(),
            cfg_parent,
            target_dir.as_path(),
            binary_name,
            stdout,
            stderr,
            env_vars,
            cargo_args,
            arch,
        )
    });

    // Cleanup regardless of errors
    docker_cleanup(container_name, target_dir.as_path())?;

    // Done.
    result
}

fn docker_prep(container_name: &str, build_config: &BuildConfig) -> Result<()> {
    // Set the solana version in the container, if given. Otherwise use the
    // default.
    match build_config.bootstrap {
        BootstrapMode::Debian => {
            // Install build requirements
            docker_exec(container_name, &["apt", "update"])?;
            docker_exec(
                container_name,
                &["apt", "install", "-y", "curl", "build-essential"],
            )?;

            // Install Rust
            docker_exec(
                container_name,
                &["curl", "https://sh.rustup.rs", "-sfo", "rustup.sh"],
            )?;
            docker_exec(container_name, &["sh", "rustup.sh", "-y"])?;
            docker_exec(container_name, &["rm", "-f", "rustup.sh"])?;
        }
        BootstrapMode::None => {}
    }

    if let Some(solana_version) = &build_config.solana_version {
        println!("Using solana version: {solana_version}");

        // Install Solana CLI
        docker_exec(
            container_name,
            &[
                "curl",
                "-sSfL",
                &format!("https://release.solana.com/v{solana_version}/install",),
                "-o",
                "solana_installer.sh",
            ],
        )?;
        docker_exec(container_name, &["sh", "solana_installer.sh"])?;
        docker_exec(container_name, &["rm", "-f", "solana_installer.sh"])?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn docker_build_bpf(
    container_name: &str,
    cargo_toml: &Path,
    cfg_parent: &Path,
    target_dir: &Path,
    binary_name: String,
    stdout: Option<File>,
    stderr: Option<File>,
    env_vars: Vec<String>,
    cargo_args: Vec<String>,
    arch: &ProgramArch,
) -> Result<()> {
    let manifest_path =
        pathdiff::diff_paths(cargo_toml.canonicalize()?, cfg_parent.canonicalize()?)
            .ok_or_else(|| anyhow!("Unable to diff paths"))?;
    println!(
        "Building {} manifest: {:?}",
        binary_name,
        manifest_path.display()
    );

    let subcommand = arch.build_subcommand();

    // Execute the build.
    let exit = std::process::Command::new("docker")
        .args([
            "exec",
            "--env",
            "PATH=/root/.local/share/solana/install/active_release/bin:/root/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
        ])
        .args(env_vars
            .iter()
            .map(|x| ["--env", x.as_str()])
            .collect::<Vec<[&str; 2]>>()
            .concat())
        .args([
            container_name,
            "cargo",
            subcommand,
            "--manifest-path",
            &manifest_path.display().to_string(),
        ])
        .args(cargo_args)
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
    let out_file = cfg_parent
        .canonicalize()?
        .join(format!("target/verifiable/{binary_name}.so"))
        .display()
        .to_string();

    // This requires the target directory of any built program to be located at
    // the root of the workspace.
    let mut bin_path = target_dir.join("deploy");
    bin_path.push(format!("{binary_name}.so"));
    let bin_artifact = format!(
        "{}:{}",
        container_name,
        bin_path.as_path().to_str().unwrap()
    );
    let exit = std::process::Command::new("docker")
        .args(["cp", &bin_artifact, &out_file])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| anyhow::format_err!("{}", e.to_string()))?;
    if !exit.status.success() {
        Err(anyhow!(
            "Failed to copy binary out of docker. Is the target directory set correctly?"
        ))
    } else {
        Ok(())
    }
}

fn docker_cleanup(container_name: &str, target_dir: &Path) -> Result<()> {
    // Wipe the generated docker-target dir.
    println!("Cleaning up the docker target directory");
    docker_exec(container_name, &["rm", "-rf", target_dir.to_str().unwrap()])?;

    // Remove the docker image.
    println!("Removing the docker container");
    let exit = std::process::Command::new("docker")
        .args(["rm", "-f", container_name])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| anyhow::format_err!("{}", e.to_string()))?;
    if !exit.status.success() {
        println!("Unable to remove the docker container");
        std::process::exit(exit.status.code().unwrap_or(1));
    }
    Ok(())
}

fn docker_exec(container_name: &str, args: &[&str]) -> Result<()> {
    let exit = std::process::Command::new("docker")
        .args([&["exec", container_name], args].concat())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| anyhow!("Failed to run command \"{:?}\": {:?}", args, e))?;
    if !exit.status.success() {
        Err(anyhow!("Failed to run command: {:?}", args))
    } else {
        Ok(())
    }
}

fn _build_rust_cwd(
    cfg: &WithPath<Config>,
    idl_out: Option<PathBuf>,
    idl_ts_out: Option<PathBuf>,
    skip_lint: bool,
    arch: &ProgramArch,
    cargo_args: Vec<String>,
) -> Result<()> {
    let subcommand = arch.build_subcommand();
    let exit = std::process::Command::new("cargo")
        .arg(subcommand)
        .args(cargo_args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| anyhow::format_err!("{}", e.to_string()))?;
    if !exit.status.success() {
        std::process::exit(exit.status.code().unwrap_or(1));
    }

    // Always assume idl is located at src/lib.rs.
    if let Some(idl) = extract_idl(cfg, "src/lib.rs", skip_lint, false)? {
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
        fs::write(&ts_out, rust_template::idl_ts(&idl)?)?;
        // Copy out the TypeScript type.
        let cfg_parent = cfg.path().parent().expect("Invalid Anchor.toml");
        if !&cfg.workspace.types.is_empty() {
            fs::copy(
                &ts_out,
                cfg_parent
                    .join(&cfg.workspace.types)
                    .join(&idl.name)
                    .with_extension("ts"),
            )?;
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn _build_solidity_cwd(
    cfg: &WithPath<Config>,
    name: &str,
    path: &Path,
    idl_out: Option<PathBuf>,
    idl_ts_out: Option<PathBuf>,
    stdout: Option<File>,
    stderr: Option<File>,
    solang_args: Vec<String>,
) -> Result<()> {
    let mut cmd = std::process::Command::new("solang");
    let cmd = cmd.args(["compile", "--target", "solana", "--contract", name]);

    if let Some(idl_out) = &idl_out {
        cmd.arg("--output-meta");
        cmd.arg(idl_out);
    }

    let target_bin = cfg.path().parent().unwrap().join("target").join("deploy");

    cmd.arg("--output");
    cmd.arg(target_bin);
    cmd.arg("--verbose");
    cmd.arg(path);

    let exit = cmd
        .args(solang_args)
        .stdout(match stdout {
            None => Stdio::inherit(),
            Some(f) => f.into(),
        })
        .stderr(match stderr {
            None => Stdio::inherit(),
            Some(f) => f.into(),
        })
        .output()
        .map_err(|e| anyhow::format_err!("{}", e.to_string()))?;
    if !exit.status.success() {
        std::process::exit(exit.status.code().unwrap_or(1));
    }

    // idl is written to idl_out or .
    let idl_path = idl_out
        .unwrap_or(PathBuf::from("."))
        .join(format!("{}.json", name));

    let idl = fs::read_to_string(idl_path)?;

    let idl: Idl = serde_json::from_str(&idl)?;

    // TS out path.
    let ts_out = match idl_ts_out {
        None => PathBuf::from(".").join(&idl.name).with_extension("ts"),
        Some(o) => PathBuf::from(&o.join(&idl.name).with_extension("ts")),
    };

    // Write out the TypeScript type.
    fs::write(&ts_out, rust_template::idl_ts(&idl)?)?;
    // Copy out the TypeScript type.
    let cfg_parent = cfg.path().parent().expect("Invalid Anchor.toml");
    if !&cfg.workspace.types.is_empty() {
        fs::copy(
            &ts_out,
            cfg_parent
                .join(&cfg.workspace.types)
                .join(&idl.name)
                .with_extension("ts"),
        )?;
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn verify(
    cfg_override: &ConfigOverride,
    program_id: Pubkey,
    program_name: Option<String>,
    solana_version: Option<String>,
    docker_image: Option<String>,
    bootstrap: BootstrapMode,
    env_vars: Vec<String>,
    cargo_args: Vec<String>,
    skip_build: bool,
    arch: ProgramArch,
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
    if !skip_build {
        build(
            cfg_override,
            None,                                                  // idl
            None,                                                  // idl ts
            true,                                                  // verifiable
            true,                                                  // skip lint
            None,                                                  // program name
            solana_version.or_else(|| cfg.solana_version.clone()), // solana version
            docker_image,                                          // docker image
            bootstrap,                                             // bootstrap docker image
            None,                                                  // stdout
            None,                                                  // stderr
            env_vars,
            cargo_args,
            false,
            arch,
        )?;
    }
    std::env::set_current_dir(cur_dir)?;

    // Verify binary.
    let binary_name = cargo.lib_name()?;
    let bin_path = cfg
        .path()
        .parent()
        .ok_or_else(|| anyhow!("Unable to find workspace root"))?
        .join("target/verifiable/")
        .join(format!("{binary_name}.so"));

    let url = cluster_url(&cfg, &cfg.test_validator);
    let bin_ver = verify_bin(program_id, &bin_path, &url)?;
    if !bin_ver.is_verified {
        println!("Error: Binaries don't match");
        std::process::exit(1);
    }

    // Verify IDL (only if it's not a buffer account).
    if let Some(local_idl) = extract_idl(&cfg, "src/lib.rs", true, false)? {
        if bin_ver.state != BinVerificationState::Buffer {
            let deployed_idl = fetch_idl(cfg_override, program_id)?;
            if local_idl != deployed_idl {
                println!("Error: IDLs don't match");
                std::process::exit(1);
            }
        }
    }

    println!("{program_id} is verified.");

    Ok(())
}

fn cd_member(cfg_override: &ConfigOverride, program_name: &str) -> Result<()> {
    // Change directories to the given `program_name`, if given.
    let cfg = Config::discover(cfg_override)?.expect("Not in workspace.");

    for program in cfg.read_all_programs()? {
        if program.solidity {
            if let Some(path) = program.path.parent() {
                std::env::set_current_dir(path)?;
                return Ok(());
            }
        } else {
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
    }
    Err(anyhow!("{} is not part of the workspace", program_name,))
}

pub fn verify_bin(program_id: Pubkey, bin_path: &Path, cluster: &str) -> Result<BinVerification> {
    // Use `finalized` state for verify
    let client = RpcClient::new_with_commitment(cluster, CommitmentConfig::finalized());

    // Get the deployed build artifacts.
    let (deployed_bin, state) = {
        let account = client.get_account(&program_id)?;
        if account.owner == bpf_loader::id() || account.owner == bpf_loader_deprecated::id() {
            let bin = account.data.to_vec();
            let state = BinVerificationState::ProgramData {
                slot: 0, // Need to look through the transaction history.
                upgrade_authority_address: None,
            };
            (bin, state)
        } else if account.owner == bpf_loader_upgradeable::id() {
            match account.state()? {
                UpgradeableLoaderState::Program {
                    programdata_address,
                } => {
                    let account = client.get_account(&programdata_address)?;
                    let bin = account.data
                        [UpgradeableLoaderState::size_of_programdata_metadata()..]
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
                    let offset = UpgradeableLoaderState::size_of_buffer_metadata();
                    (
                        account.data[offset..].to_vec(),
                        BinVerificationState::Buffer,
                    )
                }
                _ => {
                    return Err(anyhow!(
                        "Invalid program id, not a buffer or program account"
                    ))
                }
            }
        } else {
            return Err(anyhow!(
                "Invalid program id, not owned by any loader program"
            ));
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

#[derive(PartialEq, Eq)]
pub struct BinVerification {
    pub state: BinVerificationState,
    pub is_verified: bool,
}

#[derive(PartialEq, Eq)]
pub enum BinVerificationState {
    Buffer,
    ProgramData {
        slot: u64,
        upgrade_authority_address: Option<Pubkey>,
    },
}

// Fetches an IDL for the given program_id.
fn fetch_idl(cfg_override: &ConfigOverride, idl_addr: Pubkey) -> Result<Idl> {
    let url = match Config::discover(cfg_override)? {
        Some(cfg) => cluster_url(&cfg, &cfg.test_validator),
        None => {
            // If the command is not run inside a workspace,
            // cluster_url will be used from default solana config
            // provider.cluster option can be used to override this

            if let Some(cluster) = cfg_override.cluster.clone() {
                cluster.url().to_string()
            } else {
                config::get_solana_cfg_url()?
            }
        }
    };

    let client = create_client(url);

    let mut account = client.get_account(&idl_addr)?;
    if account.executable {
        let idl_addr = IdlAccount::address(&idl_addr);
        account = client.get_account(&idl_addr)?;
    }

    // Cut off account discriminator.
    let mut d: &[u8] = &account.data[8..];
    let idl_account: IdlAccount = AnchorDeserialize::deserialize(&mut d)?;

    let compressed_len: usize = idl_account.data_len.try_into().unwrap();
    let compressed_bytes = &account.data[44..44 + compressed_len];
    let mut z = ZlibDecoder::new(compressed_bytes);
    let mut s = Vec::new();
    z.read_to_end(&mut s)?;
    serde_json::from_slice(&s[..]).map_err(Into::into)
}

fn extract_idl(
    cfg: &WithPath<Config>,
    file: &str,
    skip_lint: bool,
    no_docs: bool,
) -> Result<Option<Idl>> {
    let file = shellexpand::tilde(file);
    let manifest_from_path = std::env::current_dir()?.join(PathBuf::from(&*file).parent().unwrap());
    let cargo = Manifest::discover_from_path(manifest_from_path)?
        .ok_or_else(|| anyhow!("Cargo.toml not found"))?;
    anchor_syn::idl::file::parse(
        &*file,
        cargo.version(),
        cfg.features.seeds,
        no_docs,
        !(cfg.features.skip_lint || skip_lint),
    )
}

fn idl(cfg_override: &ConfigOverride, subcmd: IdlCommand) -> Result<()> {
    match subcmd {
        IdlCommand::Init {
            program_id,
            filepath,
        } => idl_init(cfg_override, program_id, filepath),
        IdlCommand::Close {
            program_id,
            print_only,
        } => idl_close(cfg_override, program_id, print_only),
        IdlCommand::WriteBuffer {
            program_id,
            filepath,
        } => idl_write_buffer(cfg_override, program_id, filepath).map(|_| ()),
        IdlCommand::SetBuffer {
            program_id,
            buffer,
            print_only,
        } => idl_set_buffer(cfg_override, program_id, buffer, print_only),
        IdlCommand::Upgrade {
            program_id,
            filepath,
        } => idl_upgrade(cfg_override, program_id, filepath),
        IdlCommand::SetAuthority {
            program_id,
            address,
            new_authority,
            print_only,
        } => idl_set_authority(cfg_override, program_id, address, new_authority, print_only),
        IdlCommand::EraseAuthority { program_id } => idl_erase_authority(cfg_override, program_id),
        IdlCommand::Authority { program_id } => idl_authority(cfg_override, program_id),
        IdlCommand::Parse {
            file,
            out,
            out_ts,
            no_docs,
        } => idl_parse(cfg_override, file, out, out_ts, no_docs),
        IdlCommand::Fetch { address, out } => idl_fetch(cfg_override, address, out),
    }
}

fn get_idl_account(client: &RpcClient, idl_address: &Pubkey) -> Result<IdlAccount> {
    let account = client.get_account(idl_address)?;
    let mut data: &[u8] = &account.data;
    AccountDeserialize::try_deserialize(&mut data).map_err(|e| anyhow!("{:?}", e))
}

fn idl_init(cfg_override: &ConfigOverride, program_id: Pubkey, idl_filepath: String) -> Result<()> {
    with_workspace(cfg_override, |cfg| {
        let keypair = cfg.provider.wallet.to_string();

        let bytes = fs::read(idl_filepath)?;
        let idl: Idl = serde_json::from_reader(&*bytes)?;

        let idl_address = create_idl_account(cfg, &keypair, &program_id, &idl)?;

        println!("Idl account created: {idl_address:?}");
        Ok(())
    })
}

fn idl_close(cfg_override: &ConfigOverride, program_id: Pubkey, print_only: bool) -> Result<()> {
    with_workspace(cfg_override, |cfg| {
        let idl_address = IdlAccount::address(&program_id);
        idl_close_account(cfg, &program_id, idl_address, print_only)?;

        if !print_only {
            println!("Idl account closed: {idl_address:?}");
        }

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

        println!("Idl buffer created: {idl_buffer:?}");

        Ok(idl_buffer)
    })
}

fn idl_set_buffer(
    cfg_override: &ConfigOverride,
    program_id: Pubkey,
    buffer: Pubkey,
    print_only: bool,
) -> Result<()> {
    with_workspace(cfg_override, |cfg| {
        let keypair = solana_sdk::signature::read_keypair_file(&cfg.provider.wallet.to_string())
            .map_err(|_| anyhow!("Unable to read keypair file"))?;
        let url = cluster_url(cfg, &cfg.test_validator);
        let client = create_client(url);

        let idl_address = IdlAccount::address(&program_id);
        let idl_authority = if print_only {
            get_idl_account(&client, &idl_address)?.authority
        } else {
            keypair.pubkey()
        };
        // Instruction to set the buffer onto the IdlAccount.
        let ix = {
            let accounts = vec![
                AccountMeta::new(buffer, false),
                AccountMeta::new(idl_address, false),
                AccountMeta::new(idl_authority, true),
            ];
            let mut data = anchor_lang::idl::IDL_IX_TAG.to_le_bytes().to_vec();
            data.append(&mut IdlInstruction::SetBuffer.try_to_vec()?);
            Instruction {
                program_id,
                accounts,
                data,
            }
        };

        if print_only {
            print_idl_instruction("SetBuffer", &ix, &idl_address)?;
        } else {
            // Build the transaction.
            let latest_hash = client.get_latest_blockhash()?;
            let tx = Transaction::new_signed_with_payer(
                &[ix],
                Some(&keypair.pubkey()),
                &[&keypair],
                latest_hash,
            );

            // Send the transaction.
            client.send_and_confirm_transaction_with_spinner(&tx)?;
        }

        Ok(())
    })
}

fn idl_upgrade(
    cfg_override: &ConfigOverride,
    program_id: Pubkey,
    idl_filepath: String,
) -> Result<()> {
    let buffer = idl_write_buffer(cfg_override, program_id, idl_filepath)?;
    idl_set_buffer(cfg_override, program_id, buffer, false)
}

fn idl_authority(cfg_override: &ConfigOverride, program_id: Pubkey) -> Result<()> {
    with_workspace(cfg_override, |cfg| {
        let url = cluster_url(cfg, &cfg.test_validator);
        let client = create_client(url);
        let idl_address = {
            let account = client.get_account(&program_id)?;
            if account.executable {
                IdlAccount::address(&program_id)
            } else {
                program_id
            }
        };

        let idl_account = get_idl_account(&client, &idl_address)?;

        println!("{:?}", idl_account.authority);

        Ok(())
    })
}

fn idl_set_authority(
    cfg_override: &ConfigOverride,
    program_id: Pubkey,
    address: Option<Pubkey>,
    new_authority: Pubkey,
    print_only: bool,
) -> Result<()> {
    with_workspace(cfg_override, |cfg| {
        // Misc.
        let idl_address = match address {
            None => IdlAccount::address(&program_id),
            Some(addr) => addr,
        };
        let keypair = solana_sdk::signature::read_keypair_file(&cfg.provider.wallet.to_string())
            .map_err(|_| anyhow!("Unable to read keypair file"))?;
        let url = cluster_url(cfg, &cfg.test_validator);
        let client = create_client(url);

        let idl_authority = if print_only {
            get_idl_account(&client, &idl_address)?.authority
        } else {
            keypair.pubkey()
        };

        // Instruction data.
        let data =
            serialize_idl_ix(anchor_lang::idl::IdlInstruction::SetAuthority { new_authority })?;

        // Instruction accounts.
        let accounts = vec![
            AccountMeta::new(idl_address, false),
            AccountMeta::new_readonly(idl_authority, true),
        ];

        // Instruction.
        let ix = Instruction {
            program_id,
            accounts,
            data,
        };

        if print_only {
            print_idl_instruction("SetAuthority", &ix, &idl_address)?;
        } else {
            // Send transaction.
            let latest_hash = client.get_latest_blockhash()?;
            let tx = Transaction::new_signed_with_payer(
                &[ix],
                Some(&keypair.pubkey()),
                &[&keypair],
                latest_hash,
            );
            client.send_and_confirm_transaction_with_spinner(&tx)?;

            println!("Authority update complete.");
        }

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

    idl_set_authority(cfg_override, program_id, None, ERASED_AUTHORITY, false)?;

    Ok(())
}

fn idl_close_account(
    cfg: &Config,
    program_id: &Pubkey,
    idl_address: Pubkey,
    print_only: bool,
) -> Result<()> {
    let keypair = solana_sdk::signature::read_keypair_file(&cfg.provider.wallet.to_string())
        .map_err(|_| anyhow!("Unable to read keypair file"))?;
    let url = cluster_url(cfg, &cfg.test_validator);
    let client = create_client(url);

    let idl_authority = if print_only {
        get_idl_account(&client, &idl_address)?.authority
    } else {
        keypair.pubkey()
    };
    // Instruction accounts.
    let accounts = vec![
        AccountMeta::new(idl_address, false),
        AccountMeta::new_readonly(idl_authority, true),
        AccountMeta::new(keypair.pubkey(), false),
    ];
    // Instruction.
    let ix = Instruction {
        program_id: *program_id,
        accounts,
        data: { serialize_idl_ix(anchor_lang::idl::IdlInstruction::Close {})? },
    };

    if print_only {
        print_idl_instruction("Close", &ix, &idl_address)?;
    } else {
        // Send transaction.
        let latest_hash = client.get_latest_blockhash()?;
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&keypair.pubkey()),
            &[&keypair],
            latest_hash,
        );
        client.send_and_confirm_transaction_with_spinner(&tx)?;
    }

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
    let url = cluster_url(cfg, &cfg.test_validator);
    let client = create_client(url);

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
        let latest_hash = client.get_latest_blockhash()?;
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&keypair.pubkey()),
            &[&keypair],
            latest_hash,
        );
        client.send_and_confirm_transaction_with_spinner(&tx)?;
        offset += MAX_WRITE_SIZE;
    }
    Ok(())
}

fn idl_parse(
    cfg_override: &ConfigOverride,
    file: String,
    out: Option<String>,
    out_ts: Option<String>,
    no_docs: bool,
) -> Result<()> {
    let cfg = Config::discover(cfg_override)?.expect("Not in workspace.");
    let idl = extract_idl(&cfg, &file, true, no_docs)?.ok_or_else(|| anyhow!("IDL not parsed"))?;
    let out = match out {
        None => OutFile::Stdout,
        Some(out) => OutFile::File(PathBuf::from(out)),
    };
    write_idl(&idl, out)?;

    // Write out the TypeScript IDL.
    if let Some(out) = out_ts {
        fs::write(out, rust_template::idl_ts(&idl)?)?;
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
        OutFile::Stdout => println!("{idl_json}"),
        OutFile::File(out) => fs::write(out, idl_json)?,
    };

    Ok(())
}

/// Print `base64+borsh` encoded IDL instruction.
fn print_idl_instruction(ix_name: &str, ix: &Instruction, idl_address: &Pubkey) -> Result<()> {
    println!("Print only mode. No execution!");
    println!("Instruction: {ix_name}");
    println!("IDL address: {idl_address}");
    println!("Program: {}", ix.program_id);

    // Serialize with `bincode` because `Instruction` does not implement `BorshSerialize`
    let mut serialized_ix = bincode::serialize(ix)?;

    // Remove extra bytes in order to make the serialized instruction `borsh` compatible
    // `bincode` uses 8 bytes(LE) for length meanwhile `borsh` uses 4 bytes(LE)
    let mut remove_extra_vec_bytes = |index: usize| {
        serialized_ix.drain((index + 4)..(index + 8));
    };

    let accounts_index = std::mem::size_of_val(&ix.program_id);
    remove_extra_vec_bytes(accounts_index);
    let data_index = accounts_index + 4 + std::mem::size_of_val(&*ix.accounts);
    remove_extra_vec_bytes(data_index);

    println!(
        "Base64 encoded instruction: {}",
        base64::encode(serialized_ix)
    );

    Ok(())
}

fn account(
    cfg_override: &ConfigOverride,
    account_type: String,
    address: Pubkey,
    idl_filepath: Option<String>,
) -> Result<()> {
    let (program_name, account_type_name) = account_type
        .split_once('.') // Split at first occurance of dot
        .and_then(|(x, y)| y.find('.').map_or_else(|| Some((x, y)), |_| None)) // ensures no dots in second substring
        .ok_or_else(|| {
            anyhow!(
                "Please enter the account struct in the following format: <program_name>.<Account>",
            )
        })?;

    let idl = idl_filepath.map_or_else(
        || {
            Config::discover(cfg_override)
                .expect("Error when detecting workspace.")
                .expect("Not in workspace.")
                .read_all_programs()
                .expect("Workspace must contain atleast one program.")
                .iter()
                .find(|&p| p.lib_name == *program_name)
                .unwrap_or_else(|| panic!("Program {program_name} not found in workspace."))
                .idl
                .as_ref()
                .expect("IDL not found. Please build the program atleast once to generate the IDL.")
                .clone()
        },
        |idl_path| {
            let bytes = fs::read(idl_path).expect("Unable to read IDL.");
            let idl: Idl = serde_json::from_reader(&*bytes).expect("Invalid IDL format.");

            if idl.name != program_name {
                panic!("IDL does not match program {program_name}.");
            }

            idl
        },
    );

    let mut cluster = &Config::discover(cfg_override)
        .map(|cfg| cfg.unwrap())
        .map(|cfg| cfg.provider.cluster.clone())
        .unwrap_or(Cluster::Localnet);
    cluster = cfg_override.cluster.as_ref().unwrap_or(cluster);

    let data = create_client(cluster.url()).get_account_data(&address)?;
    if data.len() < 8 {
        return Err(anyhow!(
            "The account has less than 8 bytes and is not an Anchor account."
        ));
    }
    let mut data_view = &data[8..];

    let deserialized_json =
        deserialize_idl_struct_to_json(&idl, account_type_name, &mut data_view)?;

    println!(
        "{}",
        serde_json::to_string_pretty(&deserialized_json).unwrap()
    );

    Ok(())
}

// Deserializes a user defined IDL struct/enum by munching the account data.
// Recursively deserializes elements one by one
fn deserialize_idl_struct_to_json(
    idl: &Idl,
    account_type_name: &str,
    data: &mut &[u8],
) -> Result<JsonValue, anyhow::Error> {
    let account_type = &idl
        .accounts
        .iter()
        .chain(idl.types.iter())
        .find(|account_type| account_type.name == account_type_name)
        .ok_or_else(|| {
            anyhow::anyhow!("Struct/Enum named {} not found in IDL.", account_type_name)
        })?
        .ty;

    let mut deserialized_fields = Map::new();

    match account_type {
        IdlTypeDefinitionTy::Struct { fields } => {
            for field in fields {
                deserialized_fields.insert(
                    field.name.clone(),
                    deserialize_idl_type_to_json(&field.ty, data, idl)?,
                );
            }
        }
        IdlTypeDefinitionTy::Enum { variants } => {
            let repr = <u8 as AnchorDeserialize>::deserialize(data)?;

            let variant = variants
                .get(repr as usize)
                .unwrap_or_else(|| panic!("Error while deserializing enum variant {repr}"));

            let mut value = json!({});

            if let Some(enum_field) = &variant.fields {
                match enum_field {
                    EnumFields::Named(fields) => {
                        let mut values = Map::new();

                        for field in fields {
                            values.insert(
                                field.name.clone(),
                                deserialize_idl_type_to_json(&field.ty, data, idl)?,
                            );
                        }

                        value = JsonValue::Object(values);
                    }
                    EnumFields::Tuple(fields) => {
                        let mut values = Vec::new();

                        for field in fields {
                            values.push(deserialize_idl_type_to_json(field, data, idl)?);
                        }

                        value = JsonValue::Array(values);
                    }
                }
            }

            deserialized_fields.insert(variant.name.clone(), value);
        }
    }

    Ok(JsonValue::Object(deserialized_fields))
}

// Deserializes a primitive type using AnchorDeserialize
fn deserialize_idl_type_to_json(
    idl_type: &IdlType,
    data: &mut &[u8],
    parent_idl: &Idl,
) -> Result<JsonValue, anyhow::Error> {
    if data.is_empty() {
        return Err(anyhow::anyhow!("Unable to parse from empty bytes"));
    }

    Ok(match idl_type {
        IdlType::Bool => json!(<bool as AnchorDeserialize>::deserialize(data)?),
        IdlType::U8 => {
            json!(<u8 as AnchorDeserialize>::deserialize(data)?)
        }
        IdlType::I8 => {
            json!(<i8 as AnchorDeserialize>::deserialize(data)?)
        }
        IdlType::U16 => {
            json!(<u16 as AnchorDeserialize>::deserialize(data)?)
        }
        IdlType::I16 => {
            json!(<i16 as AnchorDeserialize>::deserialize(data)?)
        }
        IdlType::U32 => {
            json!(<u32 as AnchorDeserialize>::deserialize(data)?)
        }
        IdlType::I32 => {
            json!(<i32 as AnchorDeserialize>::deserialize(data)?)
        }
        IdlType::F32 => json!(<f32 as AnchorDeserialize>::deserialize(data)?),
        IdlType::U64 => {
            json!(<u64 as AnchorDeserialize>::deserialize(data)?)
        }
        IdlType::I64 => {
            json!(<i64 as AnchorDeserialize>::deserialize(data)?)
        }
        IdlType::F64 => json!(<f64 as AnchorDeserialize>::deserialize(data)?),
        IdlType::U128 => {
            // TODO: Remove to_string once serde_json supports u128 deserialization
            json!(<u128 as AnchorDeserialize>::deserialize(data)?.to_string())
        }
        IdlType::I128 => {
            // TODO: Remove to_string once serde_json supports i128 deserialization
            json!(<i128 as AnchorDeserialize>::deserialize(data)?.to_string())
        }
        IdlType::U256 => todo!("Upon completion of u256 IDL standard"),
        IdlType::I256 => todo!("Upon completion of i256 IDL standard"),
        IdlType::Bytes => JsonValue::Array(
            <Vec<u8> as AnchorDeserialize>::deserialize(data)?
                .iter()
                .map(|i| json!(*i))
                .collect(),
        ),
        IdlType::String => json!(<String as AnchorDeserialize>::deserialize(data)?),
        IdlType::PublicKey => {
            json!(<Pubkey as AnchorDeserialize>::deserialize(data)?.to_string())
        }
        IdlType::Defined(type_name) => deserialize_idl_struct_to_json(parent_idl, type_name, data)?,
        IdlType::Option(ty) => {
            let is_present = <u8 as AnchorDeserialize>::deserialize(data)?;

            if is_present == 0 {
                JsonValue::String("None".to_string())
            } else {
                deserialize_idl_type_to_json(ty, data, parent_idl)?
            }
        }
        IdlType::Vec(ty) => {
            let size: usize = <u32 as AnchorDeserialize>::deserialize(data)?
                .try_into()
                .unwrap();

            let mut vec_data: Vec<JsonValue> = Vec::with_capacity(size);

            for _ in 0..size {
                vec_data.push(deserialize_idl_type_to_json(ty, data, parent_idl)?);
            }

            JsonValue::Array(vec_data)
        }
        IdlType::Array(ty, size) => {
            let mut array_data: Vec<JsonValue> = Vec::with_capacity(*size);

            for _ in 0..*size {
                array_data.push(deserialize_idl_type_to_json(ty, data, parent_idl)?);
            }

            JsonValue::Array(array_data)
        }
    })
}

enum OutFile {
    Stdout,
    File(PathBuf),
}

// Builds, deploys, and tests all workspace programs in a single command.
#[allow(clippy::too_many_arguments)]
fn test(
    cfg_override: &ConfigOverride,
    skip_deploy: bool,
    skip_local_validator: bool,
    skip_build: bool,
    skip_lint: bool,
    detach: bool,
    tests_to_run: Vec<String>,
    extra_args: Vec<String>,
    env_vars: Vec<String>,
    cargo_args: Vec<String>,
    arch: ProgramArch,
) -> Result<()> {
    let test_paths = tests_to_run
        .iter()
        .map(|path| {
            PathBuf::from(path)
                .canonicalize()
                .map_err(|_| anyhow!("Wrong path {}", path))
        })
        .collect::<Result<Vec<_>, _>>()?;

    with_workspace(cfg_override, |cfg| {
        // Build if needed.
        if !skip_build {
            build(
                cfg_override,
                None,
                None,
                false,
                skip_lint,
                None,
                None,
                None,
                BootstrapMode::None,
                None,
                None,
                env_vars,
                cargo_args,
                false,
                arch,
            )?;
        }

        let root = cfg.path().parent().unwrap().to_owned();
        cfg.add_test_config(root, test_paths)?;

        // Run the deploy against the cluster in two cases:
        //
        // 1. The cluster is not localnet.
        // 2. The cluster is localnet, but we're not booting a local validator.
        //
        // In either case, skip the deploy if the user specifies.
        let is_localnet = cfg.provider.cluster == Cluster::Localnet;
        if (!is_localnet || skip_local_validator) && !skip_deploy {
            deploy(cfg_override, None, None)?;
        }
        let mut is_first_suite = true;
        if cfg.scripts.get("test").is_some() {
            is_first_suite = false;
            println!("\nFound a 'test' script in the Anchor.toml. Running it as a test suite!");
            run_test_suite(
                cfg.path(),
                cfg,
                is_localnet,
                skip_local_validator,
                skip_deploy,
                detach,
                &cfg.test_validator,
                &cfg.scripts,
                &extra_args,
            )?;
        }
        if let Some(test_config) = &cfg.test_config {
            for test_suite in test_config.iter() {
                if !is_first_suite {
                    std::thread::sleep(std::time::Duration::from_millis(
                        test_suite
                            .1
                            .test
                            .as_ref()
                            .map(|val| val.shutdown_wait)
                            .unwrap_or(SHUTDOWN_WAIT) as u64,
                    ));
                } else {
                    is_first_suite = false;
                }

                run_test_suite(
                    test_suite.0,
                    cfg,
                    is_localnet,
                    skip_local_validator,
                    skip_deploy,
                    detach,
                    &test_suite.1.test,
                    &test_suite.1.scripts,
                    &extra_args,
                )?;
            }
        }
        Ok(())
    })
}

#[allow(clippy::too_many_arguments)]
fn run_test_suite(
    test_suite_path: impl AsRef<Path>,
    cfg: &WithPath<Config>,
    is_localnet: bool,
    skip_local_validator: bool,
    skip_deploy: bool,
    detach: bool,
    test_validator: &Option<TestValidator>,
    scripts: &ScriptsConfig,
    extra_args: &[String],
) -> Result<()> {
    println!("\nRunning test suite: {:#?}\n", test_suite_path.as_ref());
    // Start local test validator, if needed.
    let mut validator_handle = None;
    if is_localnet && (!skip_local_validator) {
        let flags = match skip_deploy {
            true => None,
            false => Some(validator_flags(cfg, test_validator)?),
        };
        validator_handle = Some(start_test_validator(cfg, test_validator, flags, true)?);
    }

    let url = cluster_url(cfg, test_validator);

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
        let cmd = scripts
            .get("test")
            .expect("Not able to find script for `test`")
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
            println!("Failed to run test: {err:#}");
            return Err(err);
        }
    }

    Ok(())
}
// Returns the solana-test-validator flags. This will embed the workspace
// programs in the genesis block so we don't have to deploy every time. It also
// allows control of other solana-test-validator features.
fn validator_flags(
    cfg: &WithPath<Config>,
    test_validator: &Option<TestValidator>,
) -> Result<Vec<String>> {
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

    if let Some(test) = test_validator.as_ref() {
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
        if let Some(validator) = &test.validator {
            let entries = serde_json::to_value(validator)?;
            for (key, value) in entries.as_object().unwrap() {
                if key == "ledger" {
                    // Ledger flag is a special case as it is passed separately to the rest of
                    // these validator flags.
                    continue;
                };
                if key == "account" {
                    for entry in value.as_array().unwrap() {
                        // Push the account flag for each array entry
                        flags.push("--account".to_string());
                        flags.push(entry["address"].as_str().unwrap().to_string());
                        flags.push(entry["filename"].as_str().unwrap().to_string());
                    }
                } else if key == "account_dir" {
                    for entry in value.as_array().unwrap() {
                        flags.push("--account-dir".to_string());
                        flags.push(entry["directory"].as_str().unwrap().to_string());
                    }
                } else if key == "clone" {
                    // Client for fetching accounts data
                    let client = if let Some(url) = entries["url"].as_str() {
                        create_client(url)
                    } else {
                        return Err(anyhow!(
                            "Validator url for Solana's JSON RPC should be provided in order to clone accounts from it"
                        ));
                    };

                    let mut pubkeys = value
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|entry| {
                            let address = entry["address"].as_str().unwrap();
                            Pubkey::from_str(address)
                                .map_err(|_| anyhow!("Invalid pubkey {}", address))
                        })
                        .collect::<Result<HashSet<Pubkey>>>()?;

                    let accounts_keys = pubkeys.iter().cloned().collect::<Vec<_>>();
                    let accounts = client.get_multiple_accounts(&accounts_keys)?;

                    // Check if there are program accounts
                    for (account, acc_key) in accounts.iter().zip(accounts_keys) {
                        if let Some(account) = account {
                            if account.owner == bpf_loader_upgradeable::id() {
                                let upgradable: UpgradeableLoaderState = account
                                    .deserialize_data()
                                    .map_err(|_| anyhow!("Invalid program account {}", acc_key))?;

                                if let UpgradeableLoaderState::Program {
                                    programdata_address,
                                } = upgradable
                                {
                                    pubkeys.insert(programdata_address);
                                }
                            }
                        } else {
                            return Err(anyhow!("Account {} not found", acc_key));
                        }
                    }

                    for pubkey in &pubkeys {
                        // Push the clone flag for each array entry
                        flags.push("--clone".to_string());
                        flags.push(pubkey.to_string());
                    }
                } else {
                    // Remaining validator flags are non-array types
                    flags.push(format!("--{}", key.replace('_', "-")));
                    if let serde_json::Value::String(v) = value {
                        flags.push(v.to_string());
                    } else {
                        flags.push(value.to_string());
                    }
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
        let mut file = File::open(format!("target/idl/{}.json", program.lib_name))?;
        let mut contents = vec![];
        file.read_to_end(&mut contents)?;
        let idl: Idl = serde_json::from_slice(&contents)?;
        let metadata = idl.metadata.ok_or_else(|| {
            anyhow!(
                "Metadata property not found in IDL of program: {}",
                program.lib_name
            )
        })?;
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
    if let Some(test) = config.test_validator.as_ref() {
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
    test_validator: &Option<TestValidator>,
    flags: Option<Vec<String>>,
    test_log_stdout: bool,
) -> Result<Child> {
    let (test_ledger_directory, test_ledger_log_filename) =
        test_validator_file_paths(test_validator);

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

    let rpc_url = test_validator_rpc_url(test_validator);

    let rpc_port = cfg
        .test_validator
        .as_ref()
        .and_then(|test| test.validator.as_ref().map(|v| v.rpc_port))
        .unwrap_or(solana_sdk::rpc_port::DEFAULT_RPC_PORT);
    if !portpicker::is_free(rpc_port) {
        return Err(anyhow!(
            "Your configured rpc port: {rpc_port} is already in use"
        ));
    }
    let faucet_port = cfg
        .test_validator
        .as_ref()
        .and_then(|test| test.validator.as_ref().and_then(|v| v.faucet_port))
        .unwrap_or(solana_faucet::faucet::FAUCET_PORT);
    if !portpicker::is_free(faucet_port) {
        return Err(anyhow!(
            "Your configured faucet port: {faucet_port} is already in use"
        ));
    }

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
    let client = create_client(rpc_url);
    let mut count = 0;
    let ms_wait = test_validator
        .as_ref()
        .map(|test| test.startup_wait)
        .unwrap_or(STARTUP_WAIT);
    while count < ms_wait {
        let r = client.get_latest_blockhash();
        if r.is_ok() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
        count += 1;
    }
    if count == ms_wait {
        eprintln!(
            "Unable to get latest blockhash. Test validator does not look started. Check {test_ledger_log_filename} for errors. Consider increasing [test.startup_wait] in Anchor.toml."
        );
        validator_handle.kill()?;
        std::process::exit(1);
    }
    Ok(validator_handle)
}

// Return the URL that solana-test-validator should be running on given the
// configuration
fn test_validator_rpc_url(test_validator: &Option<TestValidator>) -> String {
    match test_validator {
        Some(TestValidator {
            validator: Some(validator),
            ..
        }) => format!("http://{}:{}", validator.bind_address, validator.rpc_port),
        _ => "http://localhost:8899".to_string(),
    }
}

// Setup and return paths to the solana-test-validator ledger directory and log
// files given the configuration
fn test_validator_file_paths(test_validator: &Option<TestValidator>) -> (String, String) {
    let ledger_directory = match test_validator {
        Some(TestValidator {
            validator: Some(validator),
            ..
        }) => &validator.ledger,
        _ => ".anchor/test-ledger",
    };

    if !Path::new(&ledger_directory).is_relative() {
        // Prevent absolute paths to avoid someone using / or similar, as the
        // directory gets removed
        eprintln!("Ledger directory {ledger_directory} must be relative");
        std::process::exit(1);
    }
    if Path::new(&ledger_directory).exists() {
        fs::remove_dir_all(ledger_directory).unwrap();
    }

    fs::create_dir_all(ledger_directory).unwrap();

    (
        ledger_directory.to_string(),
        format!("{ledger_directory}/test-ledger-log.txt"),
    )
}

fn cluster_url(cfg: &Config, test_validator: &Option<TestValidator>) -> String {
    let is_localnet = cfg.provider.cluster == Cluster::Localnet;
    match is_localnet {
        // Cluster is Localnet, assume the intent is to use the configuration
        // for solana-test-validator
        true => test_validator_rpc_url(test_validator),
        false => cfg.provider.cluster.url().to_string(),
    }
}

fn clean(cfg_override: &ConfigOverride) -> Result<()> {
    let cfg = Config::discover(cfg_override)?.expect("Not in workspace.");
    let cfg_parent = cfg.path().parent().expect("Invalid Anchor.toml");
    let target_dir = cfg_parent.join("target");
    let deploy_dir = target_dir.join("deploy");

    if target_dir.exists() {
        for entry in fs::read_dir(target_dir)? {
            let path = entry?.path();
            if path.is_dir() && path != deploy_dir {
                fs::remove_dir_all(&path)
                    .map_err(|e| anyhow!("Could not remove directory {}: {}", path.display(), e))?;
            } else if path.is_file() {
                fs::remove_file(&path)
                    .map_err(|e| anyhow!("Could not remove file {}: {}", path.display(), e))?;
            }
        }
    } else {
        println!("skipping target directory: not found")
    }

    if deploy_dir.exists() {
        for file in fs::read_dir(deploy_dir)? {
            let path = file?.path();
            if path.extension() != Some(&OsString::from("json")) {
                fs::remove_file(&path)
                    .map_err(|e| anyhow!("Could not remove file {}: {}", path.display(), e))?;
            }
        }
    } else {
        println!("skipping deploy directory: not found")
    }

    Ok(())
}

fn deploy(
    cfg_override: &ConfigOverride,
    program_str: Option<String>,
    program_keypair: Option<String>,
) -> Result<()> {
    // Execute the code within the workspace
    with_workspace(cfg_override, |cfg| {
        let url = cluster_url(cfg, &cfg.test_validator);
        let keypair = cfg.provider.wallet.to_string();

        // Deploy the programs.
        println!("Deploying cluster: {}", url);
        println!("Upgrade authority: {}", keypair);

        let mut program_found = true; // Flag to track if the specified program is found

        for mut program in cfg.read_all_programs()? {
            // If a program string is provided
            if let Some(single_prog_str) = &program_str {
                let program_name = program.path.file_name().unwrap().to_str().unwrap();

                // Check if the provided program string matches the program name
                if single_prog_str.as_str() != program_name {
                    program_found = false;
                } else {
                    program_found = true;
                }
            }

            if program_found {
                let binary_path = program.binary_path().display().to_string();

                println!(
                    "Deploying program {:?}...",
                    program.path.file_name().unwrap().to_str().unwrap()
                );
                println!("Program path: {}...", binary_path);

                let (program_keypair_filepath, program_id) = match &program_keypair {
                    Some(path) => (
                        path.clone(),
                        solana_sdk::signature::read_keypair_file(path)
                            .map_err(|_| anyhow!("Unable to read keypair file"))?
                            .pubkey(),
                    ),
                    None => (
                        program.keypair_file()?.path().display().to_string(),
                        program.pubkey()?,
                    ),
                };

                // Send deploy transactions using the Solana CLI
                let exit = std::process::Command::new("solana")
                    .arg("program")
                    .arg("deploy")
                    .arg("--url")
                    .arg(&url)
                    .arg("--keypair")
                    .arg(&keypair)
                    .arg("--program-id")
                    .arg(strip_workspace_prefix(program_keypair_filepath))
                    .arg(strip_workspace_prefix(binary_path))
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .output()
                    .expect("Must deploy");

                // Check if deployment was successful
                if !exit.status.success() {
                    println!("There was a problem deploying: {exit:?}.");
                    std::process::exit(exit.status.code().unwrap_or(1));
                }

                if let Some(mut idl) = program.idl.as_mut() {
                    // Add program address to the IDL.
                    idl.metadata = Some(serde_json::to_value(IdlTestMetadata {
                        address: program_id.to_string(),
                    })?);

                    // Persist it.
                    let idl_out = PathBuf::from("target/idl")
                        .join(&idl.name)
                        .with_extension("json");
                    write_idl(idl, OutFile::File(idl_out))?;
                }
            }

            // Break the loop if a specific programme is discovered and program_str is not None.
            if program_str.is_some() && program_found {
                break;
            }
        }

        // If a program string is provided but not found
        if program_str.is_some() && !program_found {
            println!("Specified program not found");
        } else {
            println!("Deploy success");
        }

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
        let url = cluster_url(cfg, &cfg.test_validator);
        let exit = std::process::Command::new("solana")
            .arg("program")
            .arg("deploy")
            .arg("--url")
            .arg(url)
            .arg("--keypair")
            .arg(&cfg.provider.wallet.to_string())
            .arg("--program-id")
            .arg(strip_workspace_prefix(program_id.to_string()))
            .arg(strip_workspace_prefix(program_filepath))
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .expect("Must deploy");
        if !exit.status.success() {
            println!("There was a problem deploying: {exit:?}.");
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
    let url = cluster_url(cfg, &cfg.test_validator);
    let client = create_client(url);
    let idl_data = serialize_idl(idl)?;

    // Run `Create instruction.
    {
        let pda_max_growth = 60_000;
        let idl_header_size = 44;
        let idl_data_len = idl_data.len() as u64;
        // We're only going to support up to 6 instructions in one transaction
        // because will anyone really have a >60kb IDL?
        if idl_data_len > pda_max_growth {
            return Err(anyhow!(
                "Your IDL is over 60kb and this isn't supported right now"
            ));
        }
        // Double for future growth.
        let data_len = (idl_data_len * 2).min(pda_max_growth - idl_header_size);

        let num_additional_instructions = data_len / 10000;
        let mut instructions = Vec::new();
        let data = serialize_idl_ix(anchor_lang::idl::IdlInstruction::Create { data_len })?;
        let program_signer = Pubkey::find_program_address(&[], program_id).0;
        let accounts = vec![
            AccountMeta::new_readonly(keypair.pubkey(), true),
            AccountMeta::new(idl_address, false),
            AccountMeta::new_readonly(program_signer, false),
            AccountMeta::new_readonly(solana_program::system_program::ID, false),
            AccountMeta::new_readonly(*program_id, false),
            AccountMeta::new_readonly(solana_program::sysvar::rent::ID, false),
        ];
        instructions.push(Instruction {
            program_id: *program_id,
            accounts,
            data,
        });

        for _ in 0..num_additional_instructions {
            let data = serialize_idl_ix(anchor_lang::idl::IdlInstruction::Resize { data_len })?;
            instructions.push(Instruction {
                program_id: *program_id,
                accounts: vec![
                    AccountMeta::new(idl_address, false),
                    AccountMeta::new_readonly(keypair.pubkey(), true),
                    AccountMeta::new_readonly(solana_program::system_program::ID, false),
                ],
                data,
            });
        }
        let latest_hash = client.get_latest_blockhash()?;
        let tx = Transaction::new_signed_with_payer(
            &instructions,
            Some(&keypair.pubkey()),
            &[&keypair],
            latest_hash,
        );
        client.send_and_confirm_transaction_with_spinner(&tx)?;
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
    let url = cluster_url(cfg, &cfg.test_validator);
    let client = create_client(url);

    let buffer = Keypair::new();

    // Creates the new buffer account with the system program.
    let create_account_ix = {
        let space = 8 + 32 + 4 + serialize_idl(idl)?.len();
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
    let latest_hash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[create_account_ix, create_buffer_ix],
        Some(&keypair.pubkey()),
        &[&keypair, &buffer],
        latest_hash,
    );

    // Send the transaction.
    client.send_and_confirm_transaction_with_spinner(&tx)?;

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

        let url = cluster_url(cfg, &cfg.test_validator);
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
                rust_template::deploy_ts_script_host(&url, &module_path.display().to_string());
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
                rust_template::deploy_js_script_host(&url, &module_path.display().to_string());
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
            println!("Workspace configuration error: {err}");
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
                    if std::env::set_current_dir(parent).is_err() {
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
        .as_ref()
        .unwrap_or_else(|| &Cluster::Devnet)
        .url();
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
        let url = cluster_url(cfg, &cfg.test_validator);
        let js_code = rust_template::node_shell(&url, &cfg.provider.wallet.to_string(), programs)?;
        let mut child = std::process::Command::new("node")
            .args(["-e", &js_code, "-i", "--experimental-repl-await"])
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

fn run(cfg_override: &ConfigOverride, script: String, script_args: Vec<String>) -> Result<()> {
    with_workspace(cfg_override, |cfg| {
        let url = cluster_url(cfg, &cfg.test_validator);
        let script = cfg
            .scripts
            .get(&script)
            .ok_or_else(|| anyhow!("Unable to find script"))?;
        let script_with_args = format!("{script} {}", script_args.join(" "));
        let exit = std::process::Command::new("bash")
            .arg("-c")
            .arg(&script_with_args)
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
    file.write_all(rust_template::credentials(&token).as_bytes())?;
    Ok(())
}

fn publish(
    cfg_override: &ConfigOverride,
    program_name: String,
    env_vars: Vec<String>,
    cargo_args: Vec<String>,
    skip_build: bool,
    arch: ProgramArch,
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
    let tarball_filename = dot_anchor.join(format!("{program_name}.tar.gz"));
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
    if Path::new("idl.json").exists() {
        println!("PACKING: idl.json");
        tar.append_path("idl.json")?;
    }

    // All workspace programs.
    for path in cfg.get_rust_program_list()? {
        let mut dirs = walkdir::WalkDir::new(path)
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
    if !skip_build {
        build(
            cfg_override,
            None,
            None,
            true,
            false,
            Some(program_name),
            None,
            None,
            BootstrapMode::None,
            None,
            None,
            env_vars,
            cargo_args,
            true,
            arch,
        )?;
    }

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
        .post(format!("{}/api/v0/build", cfg.registry.url))
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
        KeysCommand::Sync { program_name } => keys_sync(cfg_override, program_name),
    }
}

fn keys_list(cfg_override: &ConfigOverride) -> Result<()> {
    with_workspace(cfg_override, |cfg| {
        for program in cfg.read_all_programs()? {
            let pubkey = program.pubkey()?;
            println!("{}: {}", program.lib_name, pubkey);
        }
        Ok(())
    })
}

/// Sync the program's `declare_id!` pubkey with the pubkey from `target/deploy/<KEYPAIR>.json`.
fn keys_sync(cfg_override: &ConfigOverride, program_name: Option<String>) -> Result<()> {
    with_workspace(cfg_override, |cfg| {
        let programs = cfg.read_all_programs()?;
        let programs = match program_name {
            Some(program_name) => vec![programs
                .into_iter()
                .find(|program| program.lib_name == program_name)
                .ok_or_else(|| anyhow!("`{program_name}` is not found"))?],
            None => programs,
        };

        let declare_id_regex = RegexBuilder::new(r#"^(([\w]+::)*)declare_id!\("(\w*)"\)"#)
            .multi_line(true)
            .build()
            .unwrap();

        for program in programs {
            // Get the pubkey from the keypair file
            let actual_program_id = program.pubkey()?.to_string();

            // Handle declaration in program files
            let src_path = program.path.join("src");
            let files_to_check = vec![src_path.join("lib.rs"), src_path.join("id.rs")];

            for path in files_to_check {
                let mut content = match fs::read_to_string(&path) {
                    Ok(content) => content,
                    Err(_) => continue,
                };

                let incorrect_program_id = declare_id_regex
                    .captures(&content)
                    .and_then(|captures| captures.get(3))
                    .filter(|program_id_match| program_id_match.as_str() != actual_program_id);
                if let Some(program_id_match) = incorrect_program_id {
                    println!("Found incorrect program id declaration in {path:?}");

                    // Update the program id
                    content.replace_range(program_id_match.range(), &actual_program_id);
                    fs::write(&path, content)?;

                    println!("Updated to {actual_program_id}\n");
                    break;
                }
            }

            // Handle declaration in Anchor.toml
            'outer: for programs in cfg.programs.values_mut() {
                for (name, mut deployment) in programs {
                    // Skip other programs
                    if name != &program.lib_name {
                        continue;
                    }

                    if deployment.address.to_string() != actual_program_id {
                        println!("Found incorrect program id declaration in Anchor.toml for the program `{name}`");

                        // Update the program id
                        deployment.address = Pubkey::from_str(&actual_program_id).unwrap();
                        fs::write(cfg.path(), cfg.to_string())?;

                        println!("Updated to {actual_program_id}\n");
                        break 'outer;
                    }
                }
            }
        }

        println!("All program id declarations are synced.");

        Ok(())
    })
}

fn localnet(
    cfg_override: &ConfigOverride,
    skip_build: bool,
    skip_deploy: bool,
    skip_lint: bool,
    env_vars: Vec<String>,
    cargo_args: Vec<String>,
    arch: ProgramArch,
) -> Result<()> {
    with_workspace(cfg_override, |cfg| {
        // Build if needed.
        if !skip_build {
            build(
                cfg_override,
                None,
                None,
                false,
                skip_lint,
                None,
                None,
                None,
                BootstrapMode::None,
                None,
                None,
                env_vars,
                cargo_args,
                false,
                arch,
            )?;
        }

        let flags = match skip_deploy {
            true => None,
            false => Some(validator_flags(cfg, &cfg.test_validator)?),
        };

        let validator_handle = &mut start_test_validator(cfg, &cfg.test_validator, flags, false)?;

        // Setup log reader.
        let url = test_validator_rpc_url(&cfg.test_validator);
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
fn with_workspace<R>(
    cfg_override: &ConfigOverride,
    f: impl FnOnce(&mut WithPath<Config>) -> R,
) -> R {
    set_workspace_dir_or_exit();

    let mut cfg = Config::discover(cfg_override)
        .expect("Previously set the workspace dir")
        .expect("Anchor.toml must always exist");

    let r = f(&mut cfg);

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

// Remove the current workspace directory if it prefixes a string.
// This is used as a workaround for the Solana CLI using the uriparse crate to
// parse args but not handling percent encoding/decoding when using the path as
// a local filesystem path. Removing the workspace prefix handles most/all cases
// of spaces in keypair/binary paths, but this should be fixed in the Solana CLI
// and removed here.
fn strip_workspace_prefix(absolute_path: String) -> String {
    let workspace_prefix = std::env::current_dir().unwrap().display().to_string() + "/";
    absolute_path
        .strip_prefix(&workspace_prefix)
        .unwrap_or(&absolute_path)
        .into()
}

/// Create a new [`RpcClient`] with `confirmed` commitment level instead of the default(finalized).
fn create_client<U: ToString>(url: U) -> RpcClient {
    RpcClient::new_with_commitment(url, CommitmentConfig::confirmed())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "Anchor workspace name must be a valid Rust identifier.")]
    fn test_init_reserved_word() {
        init(
            &ConfigOverride {
                cluster: None,
                wallet: None,
            },
            "await".to_string(),
            true,
            false,
            false,
            false,
        )
        .unwrap();
    }

    #[test]
    #[should_panic(expected = "Anchor workspace name must be a valid Rust identifier.")]
    fn test_init_reserved_word_from_syn() {
        init(
            &ConfigOverride {
                cluster: None,
                wallet: None,
            },
            "fn".to_string(),
            true,
            false,
            false,
            false,
        )
        .unwrap();
    }

    #[test]
    #[should_panic(expected = "Anchor workspace name must be a valid Rust identifier.")]
    fn test_init_starting_with_digit() {
        init(
            &ConfigOverride {
                cluster: None,
                wallet: None,
            },
            "1project".to_string(),
            true,
            false,
            false,
            false,
        )
        .unwrap();
    }
}
