use crate::config::ProgramWorkspace;
use crate::VERSION;
use anchor_syn::idl::Idl;
use anyhow::Result;
use heck::{ToLowerCamelCase, ToSnakeCase, ToUpperCamelCase};
use solana_sdk::pubkey::Pubkey;
use std::fmt::Write;

pub fn default_program_id() -> Pubkey {
    "Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS"
        .parse()
        .unwrap()
}

pub fn devcontainer() -> &'static str {
    r#"// For format details, see https://aka.ms/devcontainer.json. For config options, see the
// README at: https://github.com/devcontainers/templates/tree/main/src/rust
{
	"name": "Light Protocol",
	"image": "ghcr.io/lightprotocol/devcontainer:main",
	"mounts": [
		// Solana keypair.
		{
			"source": "lightprotocol-solana-config-${devcontainerId}",
			"target": "/home/node/.config/solana",
			"type": "volume"
		}
	],
	// Use 'mounts' to make the cargo cache persistent in a Docker Volume.
	// "mounts": [
	// 	{
	// 		"source": "devcontainer-cargo-cache-${devcontainerId}",
	// 		"target": "/usr/local/cargo",
	// 		"type": "volume"
	// 	}
	// ]
	// Features to add to the dev container. More info: https://containers.dev/features.
	// "features": {},
	// Use 'forwardPorts' to make a list of ports inside the container available locally.
	// "forwardPorts": [],
	// Use 'postCreateCommand' to run commands after the container is created.
	// "postCreateCommand": "rustc --version",
	// Configure tool-specific properties.
	// "customizations": {},
	"remoteUser": "node"
}
"#
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

[patch.crates-io]
solana-program = { git = "https://github.com/Lightprotocol/solana", branch="v1.15" }
winnow = { git = "https://github.com/winnow-rs/winnow", tag="v0.4.1" }
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

pub fn cargo_toml_psp(name: &str) -> String {
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
anchor-lang = "0.26.0"
anchor-spl = "0.26.0"
merkle_tree_program = {{ git="https://github.com/lightprotocol/light-protocol", features = ["cpi"], rev="c8eabcc2ac56d6daa27bf7aca9ec5fa93447a3d5"}}
verifier_program_two = {{ git="https://github.com/lightprotocol/light-protocol", features = ["cpi"], rev="c8eabcc2ac56d6daa27bf7aca9ec5fa93447a3d5"}}
light-macros = "0.1.0"
light-verifier-sdk = {{ git="https://github.com/lightprotocol/light-protocol", rev="c8eabcc2ac56d6daa27bf7aca9ec5fa93447a3d5"}}
solana-program = "1.15.2"
groth16-solana = "0.0.1"
"#,
        name,
        name.to_snake_case(),
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

pub fn lib_rs_psp(name: &str) -> String {
    format!(
        r#"pub mod light_utils;
pub use light_utils::*;
pub mod processor;
pub mod verifying_key;
pub use verifying_key::*;

use crate::processor::{{
    process_psp_instruction_first, process_psp_instruction_third,
}};
use anchor_lang::prelude::*;
pub use processor::*;
declare_id!("{0}");

#[constant]
pub const PROGRAM_ID: &str = "{0}";

#[program]
pub mod {1} {{
    use anchor_lang::solana_program::keccak::hash;

    use super::*;

    /// This instruction is the first step of a shieled transaction.
    /// It creates and initializes a verifier state account to save state of a verification during
    /// computation verifying the zero-knowledge proof (ZKP). Additionally, it stores other data
    /// such as leaves, amounts, recipients, nullifiers, etc. to execute the protocol logic
    /// in the last transaction after successful ZKP verification. light_verifier_sdk::light_instruction::PspInstruction2
    pub fn psp_instruction_first<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, PspInstructionFirst<'info>>,
        inputs: Vec<u8>,
    ) -> Result<()> {{
        let inputs_des: InstructionDataPspInstructionFirst =
            InstructionDataPspInstructionFirst::try_deserialize_unchecked(
                &mut inputs.as_slice(),
            )?;
        let proof_a = [0u8; 64];
        let proof_b = [0u8; 128];
        let proof_c = [0u8; 64];
        let pool_type = [0u8; 32];
        let checked_inputs = vec![
            [
                vec![0u8],
                hash(&ctx.program_id.to_bytes()).try_to_vec()?[1..].to_vec(),
            ]
            .concat(),
            inputs_des.transaction_hash.to_vec(),
        ];
        process_psp_instruction_first(
            ctx,
            &proof_a,
            &proof_b,
            &proof_c,
            &inputs_des.public_amount_spl,
            &inputs_des.input_nullifier,
            &inputs_des.output_commitment,
            &inputs_des.public_amount_sol,
            &checked_inputs,
            &inputs_des.encrypted_utxos,
            &pool_type,
            &inputs_des.root_index,
            &inputs_des.relayer_fee,
        )
    }}

    pub fn psp_instruction_second<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, PspInstructionSecond<'info>>,
        inputs: Vec<u8>,
    ) -> Result<()> {{
        // cut off discriminator
        let vec = &inputs[8..];
        let _ = vec
            .chunks(32)
            .map(|input| {{
                ctx.accounts
                    .verifier_state
                    .checked_public_inputs
                    .push(input.to_vec())
            }})
            .collect::<Vec<_>>();
        Ok(())
    }}

    /// This instruction is the third and final step of a shielded transaction.
    /// The proof is verified with the parameters saved in the first transaction.
    /// At successful verification protocol logic is executed.
    pub fn psp_instruction_third<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, PspInstructionThird<'info>>,
        inputs: Vec<u8>,
    ) -> Result<()> {{
        let inputs_des: InstructionDataPspInstructionThird =
            InstructionDataPspInstructionThird::try_deserialize(&mut inputs.as_slice())?;

        process_psp_instruction_third(
            ctx,
            &inputs_des.proof_a_app,
            &inputs_des.proof_b_app,
            &inputs_des.proof_c_app,
            &inputs_des.proof_a,
            &inputs_des.proof_b,
            &inputs_des.proof_c,
        )
    }}

    /// Close the verifier state to reclaim rent in case the proofdata is wrong and does not verify.
    pub fn close_verifier_state<'a, 'b, 'c, 'info>(
        _ctx: Context<'a, 'b, 'c, 'info, CloseVerifierState<'info>>,
    ) -> Result<()> {{
        Ok(())
    }}
}}"#,
        default_program_id(),
        name.to_snake_case(),
    )
}

pub fn processor_rs_psp(_name: &str) -> String {
    format!(
        r#"use crate::verifying_key::VERIFYINGKEY;
use crate::PspInstructionFirst;
use crate::PspInstructionThird;
use anchor_lang::prelude::*;
use light_macros::pubkey;
use light_verifier_sdk::light_transaction::VERIFIER_STATE_SEED;
use light_verifier_sdk::{{
    light_app_transaction::AppTransaction,
    light_transaction::{{Config, Transaction}},
}};
use solana_program::sysvar;

#[derive(Clone)]
pub struct TransactionsConfig;
impl Config for TransactionsConfig {{
    /// Number of nullifiers to be inserted with the transaction.
    const NR_NULLIFIERS: usize = 4;
    /// Number of output utxos.
    const NR_LEAVES: usize = 4;
    /// ProgramId.
    const ID: Pubkey = pubkey!("{0}");
}}

pub fn process_psp_instruction_first<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, PspInstructionFirst<'info>>,
    proof_a: &'a [u8; 64],
    proof_b: &'a [u8; 128],
    proof_c: &'a [u8; 64],
    public_amount_spl: &'a [u8; 32],
    input_nullifier: &'a [[u8; 32]; 4],
    output_commitment: &'a [[u8; 32]; 4],
    public_amount_sol: &'a [u8; 32],
    checked_public_inputs: &'a Vec<Vec<u8>>,
    encrypted_utxos: &'a Vec<u8>,
    pool_type: &'a [u8; 32],
    root_index: &'a u64,
    relayer_fee: &'a u64,
) -> Result<()> {{
    let output_commitment = [
        [output_commitment[0], output_commitment[1]],
        [output_commitment[2], output_commitment[3]],
    ];
    let tx = Transaction::<2, 4, TransactionsConfig>::new(
        None,
        None,
        proof_a,
        proof_b,
        proof_c,
        public_amount_spl,
        public_amount_sol,
        checked_public_inputs,
        input_nullifier,
        &output_commitment,
        encrypted_utxos,
        *relayer_fee,
        (*root_index).try_into().unwrap(),
        pool_type, //pool_type
        None,
        &VERIFYINGKEY,
    );
    ctx.accounts.verifier_state.set_inner(tx.into());
    ctx.accounts.verifier_state.signer = *ctx.accounts.signing_address.key;
    Ok(())
}}

pub fn process_psp_instruction_third<'a, 'b, 'c, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, PspInstructionThird<'info>>,
    proof_a_app: &'a [u8; 64],
    proof_b_app: &'a [u8; 128],
    proof_c_app: &'a [u8; 64],
    proof_a_verifier: &'a [u8; 64],
    proof_b_verifier: &'a [u8; 128],
    proof_c_verifier: &'a [u8; 64],
) -> Result<()> {{
    // enforce current slot public input
    let current_slot = <Clock as sysvar::Sysvar>::get()?.slot;
    msg!(
        "{{}} > {{}}",
        current_slot,
        u64::from_be_bytes(
            ctx.accounts.verifier_state.checked_public_inputs[2][24..32]
                .try_into()
                .unwrap(),
        )
    );
    if current_slot
        < u64::from_be_bytes(
            ctx.accounts.verifier_state.checked_public_inputs[2][24..32]
                .try_into()
                .unwrap(),
        )
    {{
        panic!("Escrow still locked");
    }}
    // verify app proof
    let mut app_verifier = AppTransaction::<TransactionsConfig>::new(
        proof_a_app,
        proof_b_app,
        proof_c_app,
        ctx.accounts.verifier_state.checked_public_inputs.clone(),
        &VERIFYINGKEY,
    );

    app_verifier.verify()?;

    let (_, bump) = anchor_lang::prelude::Pubkey::find_program_address(
        &[
            ctx.accounts.signing_address.key().to_bytes().as_ref(),
            VERIFIER_STATE_SEED.as_ref(),
        ],
        ctx.program_id,
    );

    let bump = &[bump];
    let accounts = verifier_program_two::cpi::accounts::LightInstruction {{
        verifier_state: ctx.accounts.verifier_state.to_account_info().clone(),
        signing_address: ctx.accounts.signing_address.to_account_info().clone(),
        authority: ctx.accounts.authority.to_account_info().clone(),
        system_program: ctx.accounts.system_program.to_account_info().clone(),
        registered_verifier_pda: ctx
            .accounts
            .registered_verifier_pda
            .to_account_info()
            .clone(),
        program_merkle_tree: ctx.accounts.program_merkle_tree.to_account_info().clone(),
        transaction_merkle_tree: ctx
            .accounts
            .transaction_merkle_tree
            .to_account_info()
            .clone(),
        token_program: ctx.accounts.token_program.to_account_info().clone(),
        sender_spl: ctx.accounts.sender_spl.to_account_info().clone(),
        recipient_spl: ctx.accounts.recipient_spl.to_account_info().clone(),
        sender_sol: ctx.accounts.sender_sol.to_account_info().clone(),
        recipient_sol: ctx.accounts.recipient_sol.to_account_info().clone(),
        // relayer recipient and escrow will never be used in the same transaction
        relayer_recipient_sol: ctx.accounts.relayer_recipient_sol.to_account_info().clone(),
        token_authority: ctx.accounts.token_authority.to_account_info().clone(),
        log_wrapper: ctx.accounts.log_wrapper.to_account_info(),
    }};

    let seed = &ctx.accounts.signing_address.key().to_bytes();
    let domain_separation_seed = VERIFIER_STATE_SEED;
    let cpi_seed = &[seed, domain_separation_seed, bump];
    let final_seed = &[&cpi_seed[..]];
    let mut cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.verifier_program.to_account_info().clone(),
        accounts,
        final_seed,
    );
    cpi_ctx = cpi_ctx.with_remaining_accounts(ctx.remaining_accounts.to_vec());

    verifier_program_two::cpi::shielded_transfer_inputs(
        cpi_ctx,
        *proof_a_verifier,
        *proof_b_verifier,
        *proof_c_verifier,
        <Vec<u8> as TryInto<[u8; 32]>>::try_into(
            ctx.accounts.verifier_state.checked_public_inputs[1].to_vec(),
        )
        .unwrap(),
    )
}}"#,
        default_program_id(),
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

pub fn ts_package_json_psp(jest: bool, name: &str) -> String {
    if jest {
        format!(
            r#"{{
        "scripts": {{
            "lint:fix": "prettier */*.js \"*/**/*{{.js,.ts}}\" -w",
            "lint": "prettier */*.js \"*/**/*{{.js,.ts}}\" --check",
            "test": "light test --projectName {0} --programAddress {1}",
            "build": "light build --name {0}"
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
    "#,
            name,
            default_program_id()
        )
    } else {
        format!(
            r#"{{
    "scripts": {{
        "lint:fix": "prettier */*.js \"*/**/*{{.js,.ts}}\" -w",
        "lint": "prettier */*.js \"*/**/*{{.js,.ts}}\" --check",
        "test": "light test --testCommand {0} --programName {1} --programAddress {2}",
        "build": "light build --name {1}"
    }},
    "dependencies": {{
        "@coral-xyz/anchor": "^{VERSION}",
        "circomlib": "^2.0.5",
        "circomlibjs": "^0.1.7",
        "light-sdk": "git+https://github.com/lightprotocol/light-protocol.git#v0.3.0"
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
"#,
            name,
            name.to_snake_case(),
            default_program_id()
        )
    }
}

pub fn ts_mocha_psp(name: &str) -> String {
    format!(
        r#"import * as anchor from "@coral-xyz/anchor";

import * as anchor from "@coral-xyz/anchor";
import {{assert}} from "chai";
import {{
    Utxo,
    Transaction,
    TRANSACTION_MERKLE_TREE_KEY,
    TransactionParameters,
    Provider as LightProvider,
    confirmConfig,
    Action,
    TestRelayer,
    User,
    ProgramUtxoBalance,
    airdropSol,
    LOOK_UP_TABLE,
    verifierProgramStorageProgramId,
    verifierProgramTwoProgramId,
    ProgramParameters
}} from "light-sdk";
import {{
  Keypair as SolanaKeypair,
  SystemProgram,
  PublicKey,
  Keypair,
}} from "@solana/web3.js";

import {{ buildPoseidonOpt }} from "circomlibjs";
import {{ BN }} from "@coral-xyz/anchor";
import {{ IDL }} from "../target/types/{1}";
const path = require("path");

const verifierProgramId = new PublicKey(
  "Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS",
);
var POSEIDON;

const RPC_URL = "http://127.0.0.1:8899";

describe("{}", () => {{
  process.env.ANCHOR_PROVIDER_URL = RPC_URL;
  process.env.ANCHOR_WALLET = process.env.HOME + "/.config/solana/id.json";

  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.local(RPC_URL, confirmConfig);
  anchor.setProvider(provider);

  before(async () => {{
    POSEIDON = await buildPoseidonOpt();
  }});

  
it("Create and Spend Program Utxo ", async () => {{
    const wallet = Keypair.generate();
    await airdropSol({{
      provider,
      amount: 10_000_000_000,
      recipientPublicKey: wallet.publicKey,
    }});

    let relayer = new TestRelayer(wallet.publicKey, LOOK_UP_TABLE, wallet.publicKey, new BN(100000))
    await airdropSol({{provider, amount: 1_000_000_000, recipientPublicKey: Transaction.getRegisteredVerifierPda(TRANSACTION_MERKLE_TREE_KEY, verifierProgramStorageProgramId)}})
    await airdropSol({{provider, amount: 1_000_000_000, recipientPublicKey: Transaction.getRegisteredVerifierPda(TRANSACTION_MERKLE_TREE_KEY, verifierProgramTwoProgramId)}})

    // The light provider is a connection and wallet abstraction.
    // The wallet is used to derive the seed for your shielded keypair with a signature.
    var lightProvider = await LightProvider.init({{ wallet, url: RPC_URL, relayer }});
    lightProvider.addVerifierProgramPublickeyToLookUpTable(TransactionParameters.getVerifierProgramId(IDL));

    const user: User = await User.init({{ provider: lightProvider }});

    const outputUtxoSol = new Utxo({{
      poseidon: POSEIDON,
      assets: [SystemProgram.programId],
      account: user.account,
      amounts: [new BN(1_000_000)],
      appData: {{ releaseSlot: new BN(1) }},
      appDataIdl: IDL,
      verifierAddress: verifierProgramId,
      assetLookupTable: lightProvider.lookUpTables.assetLookupTable,
      verifierProgramLookupTable: lightProvider.lookUpTables.verifierProgramLookupTable
    }});

    const testInputsShield = {{
      utxo: outputUtxoSol,
      action: Action.SHIELD,
    }};

    let storeTransaction = await user.storeAppUtxo({{
      appUtxo: testInputsShield.utxo,
      action: testInputsShield.action,
    }});
    console.log("store program utxo transaction hash ", storeTransaction.txHash);

    const programUtxoBalance: Map<string, ProgramUtxoBalance> =
      await user.syncStorage(IDL);
    const shieldedUtxoCommitmentHash =
      testInputsShield.utxo.getCommitment(POSEIDON);
    const inputUtxo = programUtxoBalance
      .get(verifierProgramId.toBase58())
      .tokenBalances.get(testInputsShield.utxo.assets[1].toBase58())
      .utxos.get(shieldedUtxoCommitmentHash);

    Utxo.equal(POSEIDON, inputUtxo, testInputsShield.utxo, true);

    const circuitPath = path.join("build-circuit");

    const programParameters: ProgramParameters = {{
      inputs: {{
        releaseSlot: inputUtxo.appData.releaseSlot,
        currentSlot: inputUtxo.appData.releaseSlot // for testing we can use the same value
      }},
      verifierIdl: IDL,
      path: circuitPath
    }};

    let {{txHash}} = await user.executeAppUtxo({{
      appUtxo: inputUtxo,
      programParameters,
      action: Action.TRANSFER,
    }});
    console.log("transaction hash ", txHash);
    const utxoSpent = await user.getUtxo(inputUtxo.getCommitment(POSEIDON), true, IDL);
    assert.equal(utxoSpent.status, "spent");
    Utxo.equal(POSEIDON, utxoSpent.utxo, inputUtxo, true);
  }});
}});
"#,
        name.to_upper_camel_case(),
        name.to_snake_case(),
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

pub fn git_ignore_psp() -> &'static str {
    r#"
.anchor
.DS_Store
target
**/*.rs.bk
node_modules
test-ledger
.yarn
lookUpTable.txt
"#
}

pub fn prettier_ignore_psp() -> &'static str {
    r#"
.anchor
.DS_Store
target
node_modules
dist
build
test-ledger
build-circuit
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

pub fn circuit_psp(name: &str) -> String {
    format!(
        r#"pragma circom 2.1.4;
include "../node_modules/light-sdk/node_modules/circomlib/circuits/poseidon.circom";
include "../node_modules/light-sdk/circuit-lib/merkleProof.circom";
include "../node_modules/light-sdk/circuit-lib/keypair.circom";
include "../node_modules/light-sdk/node_modules/circomlib/circuits/gates.circom";
include "../node_modules/light-sdk/node_modules/circomlib/circuits/comparators.circom";

// will create a new instance of the circuit
#[instance]
{{
    fileName: {},
    config(),
    nrAppUtoxs: 1,
    publicInputs: [currentSlot]
}}

#[lightTransaction(verifierTwo)]
template {}() {{
    // defines the data which is saved in the utxo
    // this data is defined at utxo creation
    // is checked that only utxos with instructionData = hash or 0
    // exist in input utxos
    // is outside instruction
    // could add signal inputs automatically for these
    // are private inputs
    #[utxoData]
    {{
        releaseSlot
    }}
    signal input currentSlot;
    currentSlot === releaseSlot;
}}
// throw error when no utxoData -> doesn't make sense
// throw error if is declared twice
// throw error when there is no #[instance]
// throw error when there is no #[lightTransaction(verifierTwo)]
"#,
        name.to_upper_camel_case(),
        name.to_lower_camel_case()
    )
}
