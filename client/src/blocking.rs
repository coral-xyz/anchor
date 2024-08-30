use crate::{
    ClientError, Config, EventContext, EventUnsubscriber, Program, ProgramAccountsIterator,
    RequestBuilder,
};
use anchor_lang::{prelude::Pubkey, AccountDeserialize, Discriminator};
#[cfg(not(feature = "mock"))]
use solana_client::rpc_client::RpcClient;
use solana_client::{
    nonblocking::rpc_client::RpcClient as AsyncRpcClient, rpc_config::RpcSendTransactionConfig,
    rpc_filter::RpcFilterType,
};
use solana_sdk::{
    commitment_config::CommitmentConfig, signature::Signature, signer::Signer,
    transaction::Transaction,
};
use std::{marker::PhantomData, ops::Deref, sync::Arc};
use tokio::{
    runtime::{Builder, Handle},
    sync::RwLock,
};

impl<'a> EventUnsubscriber<'a> {
    /// Unsubscribe gracefully.
    pub fn unsubscribe(self) {
        self.runtime_handle.block_on(self.unsubscribe_internal())
    }
}

impl<C: Deref<Target = impl Signer> + Clone> Program<C> {
    pub fn new(
        program_id: Pubkey,
        cfg: Config<C>,
        #[cfg(feature = "mock")] rpc_client: AsyncRpcClient,
    ) -> Result<Self, ClientError> {
        let rt: tokio::runtime::Runtime = Builder::new_multi_thread().enable_all().build()?;

        #[cfg(not(feature = "mock"))]
        let rpc_client = {
            let comm_config = cfg.options.unwrap_or_default();
            let cluster_url = cfg.cluster.url().to_string();
            AsyncRpcClient::new_with_commitment(cluster_url.clone(), comm_config)
        };

        Ok(Self {
            program_id,
            cfg,
            sub_client: Arc::new(RwLock::new(None)),
            internal_rpc_client: rpc_client,
            rt,
        })
    }

    // We disable the `rpc` method for `mock` feature because otherwise we'd either have to
    // return a new `RpcClient` instance (which is different to the one used internally)
    // or require the user to pass another one in for blocking (since we use the non-blocking one under the hood).
    // The former of these would be confusing and the latter would be very annoying, especially since a user
    // using the mock feature likely already has a `RpcClient` instance at hand anyway.
    #[cfg(not(feature = "mock"))]
    pub fn rpc(&self) -> RpcClient {
        RpcClient::new_with_commitment(
            self.cfg.cluster.url().to_string(),
            self.cfg.options.unwrap_or_default(),
        )
    }

    /// Returns a request builder.
    pub fn request(&self) -> RequestBuilder<'_, C, Box<dyn Signer + '_>> {
        RequestBuilder::from(
            self.program_id,
            self.cfg.cluster.url(),
            self.cfg.payer.clone(),
            self.cfg.options,
            #[cfg(not(feature = "async"))]
            self.rt.handle(),
            &self.internal_rpc_client,
        )
    }

    /// Returns the account at the given address.
    pub fn account<T: AccountDeserialize>(&self, address: Pubkey) -> Result<T, ClientError> {
        self.rt.block_on(self.account_internal(address))
    }

    /// Returns all program accounts of the given type matching the given filters
    pub fn accounts<T: AccountDeserialize + Discriminator>(
        &self,
        filters: Vec<RpcFilterType>,
    ) -> Result<Vec<(Pubkey, T)>, ClientError> {
        self.accounts_lazy(filters)?.collect()
    }

    /// Returns all program accounts of the given type matching the given filters as an iterator
    /// Deserialization is executed lazily
    pub fn accounts_lazy<T: AccountDeserialize + Discriminator>(
        &self,
        filters: Vec<RpcFilterType>,
    ) -> Result<ProgramAccountsIterator<T>, ClientError> {
        self.rt.block_on(self.accounts_lazy_internal(filters))
    }

    pub fn on<T: anchor_lang::Event + anchor_lang::AnchorDeserialize>(
        &self,
        f: impl Fn(&EventContext, T) + Send + 'static,
    ) -> Result<EventUnsubscriber, ClientError> {
        let (handle, rx) = self.rt.block_on(self.on_internal(f))?;

        Ok(EventUnsubscriber {
            handle,
            rx,
            runtime_handle: self.rt.handle(),
            _lifetime_marker: PhantomData,
        })
    }
}

impl<'a, C: Deref<Target = impl Signer> + Clone> RequestBuilder<'a, C, Box<dyn Signer + 'a>> {
    pub fn from(
        program_id: Pubkey,
        cluster: &str,
        payer: C,
        options: Option<CommitmentConfig>,
        handle: &'a Handle,
        rpc_client: &'a AsyncRpcClient,
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
            handle,
            internal_rpc_client: rpc_client,
            _phantom: PhantomData,
        }
    }

    #[must_use]
    pub fn signer<T: Signer + 'a>(mut self, signer: T) -> Self {
        self.signers.push(Box::new(signer));
        self
    }

    pub fn signed_transaction(&self) -> Result<Transaction, ClientError> {
        self.handle.block_on(self.signed_transaction_internal())
    }

    pub fn send(&self) -> Result<Signature, ClientError> {
        self.handle.block_on(self.send_internal())
    }

    pub fn send_with_spinner_and_config(
        &self,
        config: RpcSendTransactionConfig,
    ) -> Result<Signature, ClientError> {
        self.handle
            .block_on(self.send_with_spinner_and_config_internal(config))
    }
}
