use crate::{
    ClientError, Config, EventContext, EventUnsubscriber, Program, ProgramAccountsIterator,
    RequestBuilder,
};
use anchor_lang::{prelude::Pubkey, AccountDeserialize, Discriminator};
use solana_client::{
    nonblocking::rpc_client::RpcClient as AsyncRpcClient, rpc_config::RpcSendTransactionConfig,
    rpc_filter::RpcFilterType,
};
use solana_sdk::{
    commitment_config::CommitmentConfig, hash::Hash, signature::Signature, signer::Signer,
    transaction::Transaction,
};
use std::{marker::PhantomData, ops::Deref, sync::Arc};
use tokio::sync::RwLock;

impl<'a> EventUnsubscriber<'a> {
    /// Unsubscribe gracefully.
    pub async fn unsubscribe(self) {
        self.unsubscribe_internal().await
    }
}

pub trait ThreadSafeSigner: Signer + Send + Sync + 'static {
    fn as_signer(&self) -> &dyn Signer;
}

impl<T: Signer + Send + Sync + 'static> ThreadSafeSigner for T {
    fn as_signer(&self) -> &dyn Signer {
        self
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

    /// Returns a threadsafe request builder
    pub fn request(&self) -> RequestBuilder<'_, C, Arc<dyn ThreadSafeSigner>> {
        RequestBuilder::from(
            self.program_id,
            self.cfg.cluster.url(),
            self.cfg.payer.clone(),
            self.cfg.options,
        )
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

impl<'a, C: Deref<Target = impl Signer> + Clone> RequestBuilder<'a, C, Arc<dyn ThreadSafeSigner>> {
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
            signers: Vec::new(),
            _phantom: PhantomData,
        }
    }

    #[must_use]
    pub fn signer<T: ThreadSafeSigner>(mut self, signer: T) -> Self {
        self.signers.push(Arc::new(signer));
        self
    }

    fn signed_transaction_with_blockhash(
        &self,
        latest_hash: Hash,
    ) -> Result<Transaction, ClientError> {
        let instructions = self.instructions()?;
        let signers: Vec<&dyn Signer> = self.signers.iter().map(|s| s.as_signer()).collect();
        let mut all_signers = signers;
        all_signers.push(&*self.payer);

        let tx = Transaction::new_signed_with_payer(
            &instructions,
            Some(&self.payer.pubkey()),
            &all_signers,
            latest_hash,
        );

        Ok(tx)
    }

    async fn signed_transaction_internal(&self) -> Result<Transaction, ClientError> {
        let latest_hash =
            AsyncRpcClient::new_with_commitment(self.cluster.to_owned(), self.options)
                .get_latest_blockhash()
                .await?;
        let tx = self.signed_transaction_with_blockhash(latest_hash)?;

        Ok(tx)
    }

    async fn send_internal(&self) -> Result<Signature, ClientError> {
        let rpc_client = AsyncRpcClient::new_with_commitment(self.cluster.to_owned(), self.options);
        let latest_hash = rpc_client.get_latest_blockhash().await?;
        let tx = self.signed_transaction_with_blockhash(latest_hash)?;

        rpc_client
            .send_and_confirm_transaction(&tx)
            .await
            .map_err(Into::into)
    }

    async fn send_with_spinner_and_config_internal(
        &self,
        config: RpcSendTransactionConfig,
    ) -> Result<Signature, ClientError> {
        let rpc_client = AsyncRpcClient::new_with_commitment(self.cluster.to_owned(), self.options);
        let latest_hash = rpc_client.get_latest_blockhash().await?;
        let tx = self.signed_transaction_with_blockhash(latest_hash)?;

        rpc_client
            .send_and_confirm_transaction_with_spinner_and_config(
                &tx,
                rpc_client.commitment(),
                config,
            )
            .await
            .map_err(Into::into)
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
