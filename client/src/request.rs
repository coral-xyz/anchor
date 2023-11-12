use anchor_lang::{
    solana_program::{
        hash::Hash,
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
    },
    InstructionData, ToAccountMetas,
};
use std::ops::Deref;

#[cfg(target_arch = "wasm32")]
use solana_client_wasm::{
    solana_sdk::{
        commitment_config::CommitmentConfig,
        signature::{Signature, Signer},
        transaction::Transaction,
    },
    utils::rpc_config::RpcSendTransactionConfig,
    WasmClient,
};

#[cfg(not(target_arch = "wasm32"))]
use {
    solana_client::nonblocking::rpc_client::RpcClient,
    solana_client::rpc_config::RpcSendTransactionConfig,
    solana_sdk::{
        commitment_config::CommitmentConfig,
        signature::{Signature, Signer},
        transaction::Transaction,
    },
};

#[cfg(all(not(feature = "async"), not(target_arch = "wasm32")))]
use tokio::runtime::Handle;

use crate::ClientError;

/// `RequestBuilder` provides a builder interface to create and send
/// transactions to a cluster.
pub struct RequestBuilder<'a, C> {
    pub(crate) cluster: String,
    pub(crate) program_id: Pubkey,
    pub(crate) accounts: Vec<AccountMeta>,
    pub(crate) options: CommitmentConfig,
    pub(crate) instructions: Vec<Instruction>,
    pub(crate) payer: C,
    // Serialized instruction data for the target RPC.
    pub(crate) instruction_data: Option<Vec<u8>>,
    pub(crate) signers: Vec<&'a dyn Signer>,
    #[cfg(all(not(feature = "async"), not(target_arch = "wasm32")))]
    pub(crate) handle: &'a Handle,
}

impl<'a, C: Deref<Target = impl Signer> + Clone> RequestBuilder<'a, C> {
    #[must_use]
    pub fn payer(mut self, payer: C) -> Self {
        self.payer = payer;
        self
    }

    #[must_use]
    pub fn cluster(mut self, url: &str) -> Self {
        self.cluster = url.to_string();
        self
    }

    #[must_use]
    pub fn instruction(mut self, ix: Instruction) -> Self {
        self.instructions.push(ix);
        self
    }

    #[must_use]
    pub fn program(mut self, program_id: Pubkey) -> Self {
        self.program_id = program_id;
        self
    }

    #[must_use]
    pub fn accounts(mut self, accounts: impl ToAccountMetas) -> Self {
        let mut metas = accounts.to_account_metas(None);
        self.accounts.append(&mut metas);
        self
    }

    #[must_use]
    pub fn options(mut self, options: CommitmentConfig) -> Self {
        self.options = options;
        self
    }

    #[must_use]
    pub fn args(mut self, args: impl InstructionData) -> Self {
        self.instruction_data = Some(args.data());
        self
    }

    #[must_use]
    pub fn signer(mut self, signer: &'a dyn Signer) -> Self {
        self.signers.push(signer);
        self
    }

    pub fn instructions(&self) -> Result<Vec<Instruction>, ClientError> {
        let mut instructions = self.instructions.clone();
        if let Some(ix_data) = &self.instruction_data {
            instructions.push(Instruction {
                program_id: self.program_id,
                data: ix_data.clone(),
                accounts: self.accounts.clone(),
            });
        }

        Ok(instructions)
    }

    fn signed_transaction_with_blockhash(
        &self,
        latest_hash: Hash,
    ) -> Result<Transaction, ClientError> {
        let instructions = self.instructions()?;
        let mut signers = self.signers.clone();
        signers.push(&*self.payer);

        let tx = Transaction::new_signed_with_payer(
            &instructions,
            Some(&self.payer.pubkey()),
            &signers,
            latest_hash,
        );

        Ok(tx)
    }

    pub fn transaction(&self) -> Result<Transaction, ClientError> {
        let instructions = &self.instructions;
        let tx = Transaction::new_with_payer(instructions, Some(&self.payer.pubkey()));
        Ok(tx)
    }

    pub(crate) async fn signed_transaction_internal(&self) -> Result<Transaction, ClientError> {
        let latest_hash = self.get_rpc_client().get_latest_blockhash().await?;
        let tx = self.signed_transaction_with_blockhash(latest_hash)?;

        Ok(tx)
    }

    pub(crate) async fn send_internal(&self) -> Result<Signature, ClientError> {
        let rpc_client = self.get_rpc_client();

        let latest_hash = rpc_client.get_latest_blockhash().await?;
        let tx = self.signed_transaction_with_blockhash(latest_hash)?;

        rpc_client
            .send_and_confirm_transaction(&tx)
            .await
            .map_err(Into::into)
    }

    pub(crate) async fn send_with_spinner_and_config_internal(
        &self,
        config: RpcSendTransactionConfig,
    ) -> Result<Signature, ClientError> {
        let rpc_client = self.get_rpc_client();

        let latest_hash = rpc_client.get_latest_blockhash().await?;
        let tx = self.signed_transaction_with_blockhash(latest_hash)?;

        #[cfg(not(target_arch = "wasm32"))]
        let result = rpc_client
            .send_and_confirm_transaction_with_spinner_and_config(
                &tx,
                rpc_client.commitment(),
                config,
            )
            .await?;

        #[cfg(target_arch = "wasm32")]
        let result = rpc_client
            .send_and_confirm_transaction_with_config(&tx, rpc_client.commitment_config(), config)
            .await?;

        Ok(result)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn get_rpc_client(&self) -> RpcClient {
        RpcClient::new_with_commitment(self.cluster.to_owned(), self.options)
    }

    #[cfg(target_arch = "wasm32")]
    fn get_rpc_client(&self) -> WasmClient {
        WasmClient::new_with_commitment(&self.cluster, self.options)
    }
}
