//! `anchor_client` provides an RPC client to send transactions and fetch
//! deserialized accounts from Solana programs written in `anchor_lang`.

use anchor_lang::solana_program::{program_error::ProgramError, pubkey::Pubkey};
use regex::Regex;
use std::{iter::Map, ops::Deref, sync::Arc, vec::IntoIter};
use thiserror::Error;

#[cfg(target_arch = "wasm32")]
use solana_client_wasm::{
    solana_sdk::{account::Account, commitment_config::CommitmentConfig, signature::Signer},
    utils::rpc_response::{RpcLogsResponse, WithContext as RpcResponse},
    ClientError as SolanaClientError,
};

#[cfg(not(target_arch = "wasm32"))]
use {
    futures::Future,
    solana_client::{
        client_error::ClientError as SolanaClientError,
        nonblocking::pubsub_client::PubsubClientError,
        rpc_response::{Response as RpcResponse, RpcLogsResponse},
    },
    solana_sdk::{account::Account, commitment_config::CommitmentConfig, signature::Signer},
    std::pin::Pin,
};

pub use anchor_lang;
pub use cluster::Cluster;

#[cfg(target_arch = "wasm32")]
pub use {solana_client_wasm, solana_client_wasm::solana_sdk};

#[cfg(not(target_arch = "wasm32"))]
pub use {solana_client, solana_sdk};

mod cluster;
mod event;
mod program;
mod request;

pub use event::*;
pub use program::*;
pub use request::*;

#[cfg(all(not(feature = "async"), not(target_arch = "wasm32")))]
mod blocking;
#[cfg(feature = "async")]
mod nonblocking;

const PROGRAM_LOG: &str = "Program log: ";
const PROGRAM_DATA: &str = "Program data: ";

#[cfg(all(not(feature = "async"), target_arch = "wasm32"))]
compile_error!("`async` feature must be enabled for the wasm target.");

#[cfg(not(target_arch = "wasm32"))]
type UnsubscribeFn = Box<dyn FnOnce() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send>;

/// Client defines the base configuration for building RPC clients to
/// communicate with Anchor programs running on a Solana cluster. It's
/// primary use is to build a `Program` client via the `program` method.
pub struct Client<C> {
    cfg: Config<C>,
}

impl<C: Clone + Deref<Target = impl Signer>> Client<C> {
    pub fn new(cluster: Cluster, payer: C) -> Self {
        Self {
            cfg: Config {
                cluster,
                payer,
                options: None,
            },
        }
    }

    pub fn new_with_options(cluster: Cluster, payer: C, options: CommitmentConfig) -> Self {
        Self {
            cfg: Config {
                cluster,
                payer,
                options: Some(options),
            },
        }
    }

    #[cfg(any(feature = "async", not(target_arch = "wasm32")))]
    pub fn program(&self, program_id: Pubkey) -> Result<Program<C>, ClientError> {
        let cfg = Config {
            cluster: self.cfg.cluster.clone(),
            options: self.cfg.options,
            payer: self.cfg.payer.clone(),
        };

        Program::new(program_id, cfg)
    }
}

/// Auxiliary data structure to align the types of the Solana CLI utils with Anchor client.
/// Client<C> implementation requires <C: Clone + Deref<Target = impl Signer>> which does not comply with Box<dyn Signer>
/// that's used when loaded Signer from keypair file. This struct is used to wrap the usage.
pub struct DynSigner(pub Arc<dyn Signer>);

impl Signer for DynSigner {
    fn pubkey(&self) -> Pubkey {
        self.0.pubkey()
    }

    fn try_pubkey(&self) -> Result<Pubkey, solana_sdk::signer::SignerError> {
        self.0.try_pubkey()
    }

    fn sign_message(&self, message: &[u8]) -> solana_sdk::signature::Signature {
        self.0.sign_message(message)
    }

    fn try_sign_message(
        &self,
        message: &[u8],
    ) -> Result<solana_sdk::signature::Signature, solana_sdk::signer::SignerError> {
        self.0.try_sign_message(message)
    }

    fn is_interactive(&self) -> bool {
        self.0.is_interactive()
    }
}

// Internal configuration for a client.
#[derive(Debug)]
pub struct Config<C> {
    cluster: Cluster,
    payer: C,
    options: Option<CommitmentConfig>,
}

/// Iterator with items of type (Pubkey, T). Used to lazily deserialize account structs.
/// Wrapper type hides the inner type from usages so the implementation can be changed.
pub struct ProgramAccountsIterator<T> {
    inner: Map<IntoIter<(Pubkey, Account)>, AccountConverterFunction<T>>,
}

/// Function type that accepts solana accounts and returns deserialized anchor accounts
type AccountConverterFunction<T> = fn((Pubkey, Account)) -> Result<(Pubkey, T), ClientError>;

impl<T> Iterator for ProgramAccountsIterator<T> {
    type Item = Result<(Pubkey, T), ClientError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

fn handle_program_log<T: anchor_lang::Event + anchor_lang::AnchorDeserialize>(
    self_program_str: &str,
    l: &str,
) -> Result<(Option<T>, Option<String>, bool), ClientError> {
    use anchor_lang::__private::base64;
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;

    // Log emitted from the current program.
    if let Some(log) = l
        .strip_prefix(PROGRAM_LOG)
        .or_else(|| l.strip_prefix(PROGRAM_DATA))
    {
        let borsh_bytes = match STANDARD.decode(log) {
            Ok(borsh_bytes) => borsh_bytes,
            _ => {
                #[cfg(feature = "debug")]
                println!("Could not base64 decode log: {}", log);
                return Ok((None, None, false));
            }
        };

        let mut slice: &[u8] = &borsh_bytes[..];
        let disc: [u8; 8] = {
            let mut disc = [0; 8];
            disc.copy_from_slice(&borsh_bytes[..8]);
            slice = &slice[8..];
            disc
        };
        let mut event = None;
        if disc == T::discriminator() {
            let e: T = anchor_lang::AnchorDeserialize::deserialize(&mut slice)
                .map_err(|e| ClientError::LogParseError(e.to_string()))?;
            event = Some(e);
        }
        Ok((event, None, false))
    }
    // System log.
    else {
        let (program, did_pop) = handle_system_log(self_program_str, l);
        Ok((None, program, did_pop))
    }
}

fn handle_system_log(this_program_str: &str, log: &str) -> (Option<String>, bool) {
    if log.starts_with(&format!("Program {this_program_str} log:")) {
        (Some(this_program_str.to_string()), false)
    } else if log.contains("invoke") {
        (Some("cpi".to_string()), false) // Any string will do.
    } else {
        let re = Regex::new(r"^Program (.*) success*$").unwrap();
        if re.is_match(log) {
            (None, true)
        } else {
            (None, false)
        }
    }
}

struct Execution {
    stack: Vec<String>,
}

impl Execution {
    pub fn new(logs: &mut &[String]) -> Result<Self, ClientError> {
        let l = &logs[0];
        *logs = &logs[1..];

        let re = Regex::new(r"^Program (.*) invoke.*$").unwrap();
        let c = re
            .captures(l)
            .ok_or_else(|| ClientError::LogParseError(l.to_string()))?;
        let program = c
            .get(1)
            .ok_or_else(|| ClientError::LogParseError(l.to_string()))?
            .as_str()
            .to_string();
        Ok(Self {
            stack: vec![program],
        })
    }

    pub fn program(&self) -> String {
        assert!(!self.stack.is_empty());
        self.stack[self.stack.len() - 1].clone()
    }

    pub fn push(&mut self, new_program: String) {
        self.stack.push(new_program);
    }

    pub fn pop(&mut self) {
        assert!(!self.stack.is_empty());
        self.stack.pop().unwrap();
    }
}

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("Account not found")]
    AccountNotFound,
    #[error("{0}")]
    AnchorError(#[from] anchor_lang::error::Error),
    #[error("{0}")]
    ProgramError(#[from] ProgramError),
    #[error("{0}")]
    SolanaClientError(#[from] SolanaClientError),
    #[cfg(not(target_arch = "wasm32"))]
    #[error("{0}")]
    SolanaClientPubsubError(#[from] PubsubClientError),
    #[error("Unable to parse log: {0}")]
    LogParseError(String),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}
fn parse_logs_response<T: anchor_lang::Event + anchor_lang::AnchorDeserialize>(
    logs: RpcResponse<RpcLogsResponse>,
    program_id_str: &str,
) -> Vec<T> {
    let mut logs = &logs.value.logs[..];
    let mut events: Vec<T> = Vec::new();
    if !logs.is_empty() {
        if let Ok(mut execution) = Execution::new(&mut logs) {
            for l in logs {
                // Parse the log.
                let (event, new_program, did_pop) = {
                    if program_id_str == execution.program() {
                        handle_program_log(program_id_str, l).unwrap_or_else(|e| {
                            println!("Unable to parse log: {e}");
                            std::process::exit(1);
                        })
                    } else {
                        let (program, did_pop) = handle_system_log(program_id_str, l);
                        (None, program, did_pop)
                    }
                };
                // Emit the event.
                if let Some(e) = event {
                    events.push(e);
                }
                // Switch program context on CPI.
                if let Some(new_program) = new_program {
                    execution.push(new_program);
                }
                // Program returned.
                if did_pop {
                    execution.pop();
                }
            }
        }
    }
    events
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new_execution() {
        let mut logs: &[String] =
            &["Program 7Y8VDzehoewALqJfyxZYMgYCnMTCDhWuGfJKUvjYWATw invoke [1]".to_string()];
        let exe = Execution::new(&mut logs).unwrap();
        assert_eq!(
            exe.stack[0],
            "7Y8VDzehoewALqJfyxZYMgYCnMTCDhWuGfJKUvjYWATw".to_string()
        );
    }

    #[test]
    fn handle_system_log_pop() {
        let log = "Program 7Y8VDzehoewALqJfyxZYMgYCnMTCDhWuGfJKUvjYWATw success";
        let (program, did_pop) = handle_system_log("asdf", log);
        assert_eq!(program, None);
        assert!(did_pop);
    }

    #[test]
    fn handle_system_log_no_pop() {
        let log = "Program 7swsTUiQ6KUK4uFYquQKg4epFRsBnvbrTf2fZQCa2sTJ qwer";
        let (program, did_pop) = handle_system_log("asdf", log);
        assert_eq!(program, None);
        assert!(!did_pop);
    }
}
