use crate::{
    ClientError, Config, EventContext, EventUnsubscriber, Program, ProgramAccountsIterator,
    RequestBuilder,
};
use anchor_lang::{prelude::Pubkey, AccountDeserialize, Discriminator};
use solana_client::{rpc_config::RpcSendTransactionConfig, rpc_filter::RpcFilterType};
use solana_sdk::{
    commitment_config::CommitmentConfig, signature::Signature, signer::SignerError,
    transaction::Transaction,
};
use std::{marker::PhantomData, ops::Deref, sync::Arc};
use tokio::sync::RwLock;

/// Signer which is also Send.
pub trait Signer: solana_sdk::signature::Signer + Send {}

impl<T: solana_sdk::signature::Signer + Send> Signer for T {}

/// Collection of signers.
///
/// We need a wrapper type because we cannot implement `Signers` trait on
/// `Vec<&den Signer>`.
#[derive(Clone, Default)]
pub struct Signers<'a>(pub Vec<&'a dyn Signer>);

impl<'a> Signers<'a> {
    #[inline]
    pub fn push(&mut self, signer: &'a dyn Signer) {
        self.0.push(signer)
    }
}

impl<'a> solana_sdk::signer::signers::Signers for Signers<'a> {
    #[inline]
    fn pubkeys(&self) -> Vec<Pubkey> {
        self.0.iter().map(|keypair| keypair.pubkey()).collect()
    }

    #[inline]
    fn try_pubkeys(&self) -> Result<Vec<Pubkey>, SignerError> {
        self.0.iter().map(|keypair| keypair.try_pubkey()).collect()
    }

    #[inline]
    fn sign_message(&self, message: &[u8]) -> Vec<Signature> {
        self.0
            .iter()
            .map(|keypair| keypair.sign_message(message))
            .collect()
    }

    #[inline]
    fn try_sign_message(&self, message: &[u8]) -> Result<Vec<Signature>, SignerError> {
        self.0
            .iter()
            .map(|keypair| keypair.try_sign_message(message))
            .collect()
    }

    #[inline]
    fn is_interactive(&self) -> bool {
        self.0.iter().any(|s| s.is_interactive())
    }
}

impl<'a> EventUnsubscriber<'a> {
    /// Unsubscribe gracefully.
    pub async fn unsubscribe(self) {
        self.unsubscribe_internal().await
    }
}

impl<C: Deref<Target = impl Signer> + Clone> Program<C> {
    pub fn new(program_id: Pubkey, cfg: Config<C>) -> Result<Self, ClientError> {
        Ok(Self {
            program_id,
            cfg,
            sub_client: Arc::new(RwLock::new(None)),
        })
    }

    /// Returns the account at the given address.
    pub async fn account<T: AccountDeserialize>(&self, address: Pubkey) -> Result<T, ClientError> {
        self.account_internal(address).await
    }

    /// Returns all program accounts of the given type matching the given filters
    pub async fn accounts<T: AccountDeserialize + Discriminator>(
        &self,
        filters: Vec<RpcFilterType>,
    ) -> Result<Vec<(Pubkey, T)>, ClientError> {
        self.accounts_lazy(filters).await?.collect()
    }

    /// Returns all program accounts of the given type matching the given filters as an iterator
    /// Deserialization is executed lazily
    pub async fn accounts_lazy<T: AccountDeserialize + Discriminator>(
        &self,
        filters: Vec<RpcFilterType>,
    ) -> Result<ProgramAccountsIterator<T>, ClientError> {
        self.accounts_lazy_internal(filters).await
    }

    /// Subscribe to program logs.
    ///
    /// Returns an [`EventUnsubscriber`] to unsubscribe and close connection gracefully.
    pub async fn on<T: anchor_lang::Event + anchor_lang::AnchorDeserialize>(
        &self,
        f: impl Fn(&EventContext, T) + Send + 'static,
    ) -> Result<EventUnsubscriber, ClientError> {
        let (handle, rx) = self.on_internal(f).await?;

        Ok(EventUnsubscriber {
            handle,
            rx,
            _lifetime_marker: PhantomData,
        })
    }
}

impl<'a, C: Deref<Target = impl Signer> + Clone> RequestBuilder<'a, C> {
    pub fn from(
        program_id: Pubkey,
        cluster: &str,
        payer: C,
        options: Option<CommitmentConfig>,
    ) -> Self {
        Self {
            program_id,
            payer,
            cluster: cluster.to_string(),
            accounts: Vec::new(),
            options: options.unwrap_or_default(),
            instructions: Vec::new(),
            instruction_data: None,
            signers: Default::default(),
        }
    }

    pub async fn signed_transaction(&self) -> Result<Transaction, ClientError> {
        self.signed_transaction_internal().await
    }

    pub async fn send(self) -> Result<Signature, ClientError> {
        self.send_internal().await
    }

    pub async fn send_with_spinner_and_config(
        self,
        config: RpcSendTransactionConfig,
    ) -> Result<Signature, ClientError> {
        self.send_with_spinner_and_config_internal(config).await
    }
}
