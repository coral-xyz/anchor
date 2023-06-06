use crate::config::ProgramWorkspace;
use crate::VERSION;
use anchor_syn::idl::Idl;
use anyhow::Result;
use heck::{ToLowerCamelCase, ToSnakeCase, ToUpperCamelCase};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{read_keypair_file, write_keypair_file, Keypair},
    signer::Signer,
};
use std::{fmt::Write, path::Path};

/// Read the program keypair file or create a new one if it doesn't exist.
pub fn get_or_create_program_id(name: &str) -> Pubkey {
    let keypair_path = Path::new("target")
        .join("deploy")
        .join(format!("{}-keypair.json", name.to_snake_case()));

    read_keypair_file(&keypair_path)
        .unwrap_or_else(|_| {
            let keypair = Keypair::new();
            write_keypair_file(&keypair, keypair_path).expect("Unable to create program keypair");
            keypair
        })
        .pubkey()
}

pub fn virtual_manifest() -> &'static str {
    r#"[workspace]
members = [
    "programs/*"
]

[profile.release]
overflow-checks = true
lto = "fat"
codegen-units = 1
[profile.release.build-override]
opt-level = 3
incremental = false
codegen-units = 1
"#
}

pub fn credentials(token: &str) -> String {
    format!(
        r#"[registry]
token = "{token}"
"#
    )
}

pub fn idl_ts(idl: &Idl) -> Result<String> {
    let mut idl = idl.clone();
    for acc in idl.accounts.iter_mut() {
        acc.name = acc.name.to_lower_camel_case();
    }
    let idl_json = serde_json::to_string_pretty(&idl)?;
    Ok(format!(
        r#"export type {} = {};

export const IDL: {} = {};
"#,
        idl.name.to_upper_camel_case(),
        idl_json,
        idl.name.to_upper_camel_case(),
        idl_json
    ))
}

pub fn cargo_toml(name: &str) -> String {
    format!(
        r#"[package]
name = "{0}"
version = "0.1.0"
description = "Created with Anchor"
edition = "2021"

[lib]
crate-type = ["cdylib", "lib"]
name = "{1}"

[features]
no-entrypoint = []
no-idl = []
no-log-ix-name = []
cpi = ["no-entrypoint"]
default = []

[dependencies]
anchor-lang = "{2}"
"#,
        name,
        name.to_snake_case(),
        VERSION,
    )
}

pub fn deploy_js_script_host(cluster_url: &str, script_path: &str) -> String {
    format!(
        r#"
const anchor = require('@coral-xyz/anchor');

// Deploy script defined by the user.
const userScript = require("{script_path}");

async function main() {{
    const url = "{cluster_url}";
    const preflightCommitment = 'recent';
    const connection = new anchor.web3.Connection(url, preflightCommitment);
    const wallet = anchor.Wallet.local();

    const provider = new anchor.AnchorProvider(connection, wallet, {{
        preflightCommitment,
        commitment: 'recent',
    }});

    // Run the user's deploy script.
    userScript(provider);
}}
main();
"#,
    )
}

pub fn deploy_ts_script_host(cluster_url: &str, script_path: &str) -> String {
    format!(
        r#"import * as anchor from '@coral-xyz/anchor';

// Deploy script defined by the user.
const userScript = require("{script_path}");

async function main() {{
    const url = "{cluster_url}";
    const preflightCommitment = 'recent';
    const connection = new anchor.web3.Connection(url, preflightCommitment);
    const wallet = anchor.Wallet.local();

    const provider = new anchor.AnchorProvider(connection, wallet, {{
        preflightCommitment,
        commitment: 'recent',
    }});

    // Run the user's deploy script.
    userScript(provider);
}}
main();
"#,
    )
}

pub fn deploy_script() -> &'static str {
    r#"// Migrations are an early feature. Currently, they're nothing more than this
// single deploy script that's invoked from the CLI, injecting a provider
// configured from the workspace's Anchor.toml.

const anchor = require("@coral-xyz/anchor");

module.exports = async function (provider) {
  // Configure client to use the provider.
  anchor.setProvider(provider);

  // Add your deploy script here.
};
"#
}

pub fn ts_deploy_script() -> &'static str {
    r#"// Migrations are an early feature. Currently, they're nothing more than this
// single deploy script that's invoked from the CLI, injecting a provider
// configured from the workspace's Anchor.toml.

const anchor = require("@coral-xyz/anchor");

module.exports = async function (provider) {
  // Configure client to use the provider.
  anchor.setProvider(provider);

  // Add your deploy script here.
};
"#
}

pub fn xargo_toml() -> &'static str {
    r#"[target.bpfel-unknown-unknown.dependencies.std]
features = []
"#
}

pub fn lib_rs(name: &str) -> String {
    format!(
        r#"use anchor_lang::prelude::*;

declare_id!("{}");

#[program]
pub mod {} {{
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {{
        Ok(())
    }}
}}

#[derive(Accounts)]
pub struct Initialize {{}}
"#,
        get_or_create_program_id(name),
        name.to_snake_case(),
    )
}

pub fn mocha(name: &str) -> String {
    format!(
        r#"const anchor = require("@coral-xyz/anchor");

describe("{}", () => {{
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  it("Is initialized!", async () => {{
    // Add your test here.
    const program = anchor.workspace.{};
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  }});
}});
"#,
        name,
        name.to_upper_camel_case(),
    )
}

pub fn jest(name: &str) -> String {
    format!(
        r#"const anchor = require("@coral-xyz/anchor");

describe("{}", () => {{
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  it("Is initialized!", async () => {{
    // Add your test here.
    const program = anchor.workspace.{};
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  }});
}});
"#,
        name,
        name.to_upper_camel_case(),
    )
}

pub fn package_json(jest: bool) -> String {
    if jest {
        format!(
            r#"{{
        "scripts": {{
            "lint:fix": "prettier */*.js \"*/**/*{{.js,.ts}}\" -w",
            "lint": "prettier */*.js \"*/**/*{{.js,.ts}}\" --check"
        }},
        "dependencies": {{
            "@coral-xyz/anchor": "^{VERSION}"
        }},
        "devDependencies": {{
            "jest": "^29.0.3",
            "prettier": "^2.6.2"
        }}
    }}
    "#
        )
    } else {
        format!(
            r#"{{
    "scripts": {{
        "lint:fix": "prettier */*.js \"*/**/*{{.js,.ts}}\" -w",
        "lint": "prettier */*.js \"*/**/*{{.js,.ts}}\" --check"
    }},
    "dependencies": {{
        "@coral-xyz/anchor": "^{VERSION}"
    }},
    "devDependencies": {{
        "chai": "^4.3.4",
        "mocha": "^9.0.3",
        "prettier": "^2.6.2"
    }}
}}
"#
        )
    }
}

pub fn ts_package_json(jest: bool) -> String {
    if jest {
        format!(
            r#"{{
        "scripts": {{
            "lint:fix": "prettier */*.js \"*/**/*{{.js,.ts}}\" -w",
            "lint": "prettier */*.js \"*/**/*{{.js,.ts}}\" --check"
        }},
        "dependencies": {{
            "@coral-xyz/anchor": "^{VERSION}"
        }},
        "devDependencies": {{
            "@types/bn.js": "^5.1.0",
            "@types/jest": "^29.0.3",
            "jest": "^29.0.3",
            "prettier": "^2.6.2",
            "ts-jest": "^29.0.2",
            "typescript": "^4.3.5"
        }}
    }}
    "#
        )
    } else {
        format!(
            r#"{{
    "scripts": {{
        "lint:fix": "prettier */*.js \"*/**/*{{.js,.ts}}\" -w",
        "lint": "prettier */*.js \"*/**/*{{.js,.ts}}\" --check"
    }},
    "dependencies": {{
        "@coral-xyz/anchor": "^{VERSION}"
    }},
    "devDependencies": {{
        "chai": "^4.3.4",
        "mocha": "^9.0.3",
        "ts-mocha": "^10.0.0",
        "@types/bn.js": "^5.1.0",
        "@types/chai": "^4.3.0",
        "@types/mocha": "^9.0.0",
        "typescript": "^4.3.5",
        "prettier": "^2.6.2"
    }}
}}
"#
        )
    }
}

pub fn ts_mocha(name: &str) -> String {
    format!(
        r#"import * as anchor from "@coral-xyz/anchor";
import {{ Program }} from "@coral-xyz/anchor";
import {{ {} }} from "../target/types/{}";

describe("{}", () => {{
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.{} as Program<{}>;

  it("Is initialized!", async () => {{
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  }});
}});
"#,
        name.to_upper_camel_case(),
        name.to_snake_case(),
        name,
        name.to_upper_camel_case(),
        name.to_upper_camel_case(),
    )
}

pub fn ts_jest(name: &str) -> String {
    format!(
        r#"import * as anchor from "@coral-xyz/anchor";
import {{ Program }} from "@coral-xyz/anchor";
import {{ {} }} from "../target/types/{}";

describe("{}", () => {{
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.{} as Program<{}>;

  it("Is initialized!", async () => {{
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  }});
}});
"#,
        name.to_upper_camel_case(),
        name.to_snake_case(),
        name,
        name.to_upper_camel_case(),
        name.to_upper_camel_case(),
    )
}

pub fn ts_config(jest: bool) -> &'static str {
    if jest {
        r#"{
            "compilerOptions": {
              "types": ["jest"],
              "typeRoots": ["./node_modules/@types"],
              "lib": ["es2015"],
              "module": "commonjs",
              "target": "es6",
              "esModuleInterop": true
            }
          }
          "#
    } else {
        r#"{
            "compilerOptions": {
              "types": ["mocha", "chai"],
              "typeRoots": ["./node_modules/@types"],
              "lib": ["es2015"],
              "module": "commonjs",
              "target": "es6",
              "esModuleInterop": true
            }
          }
          "#
    }
}

pub fn git_ignore() -> &'static str {
    r#"
.anchor
.DS_Store
target
**/*.rs.bk
node_modules
test-ledger
.yarn
"#
}

pub fn prettier_ignore() -> &'static str {
    r#"
.anchor
.DS_Store
target
node_modules
dist
build
test-ledger
"#
}

pub fn node_shell(
    cluster_url: &str,
    wallet_path: &str,
    programs: Vec<ProgramWorkspace>,
) -> Result<String> {
    let mut eval_string = format!(
        r#"
const anchor = require('@coral-xyz/anchor');
const web3 = anchor.web3;
const PublicKey = anchor.web3.PublicKey;
const Keypair = anchor.web3.Keypair;

const __wallet = new anchor.Wallet(
  Keypair.fromSecretKey(
    Buffer.from(
      JSON.parse(
        require('fs').readFileSync(
          "{wallet_path}",
          {{
            encoding: "utf-8",
          }},
        ),
      ),
    ),
  ),
);
const __connection = new web3.Connection("{cluster_url}", "processed");
const provider = new anchor.AnchorProvider(__connection, __wallet, {{
  commitment: "processed",
  preflightcommitment: "processed",
}});
anchor.setProvider(provider);
"#,
    );

    for program in programs {
        write!(
            &mut eval_string,
            r#"
anchor.workspace.{} = new anchor.Program({}, new PublicKey("{}"), provider);
"#,
            program.name.to_upper_camel_case(),
            serde_json::to_string(&program.idl)?,
            program.program_id
        )?;
    }

    Ok(eval_string)
}
