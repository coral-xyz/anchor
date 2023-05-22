use crate::{
    parse_logs_response, ClientError, EventContext, Program, ProgramAccountsIterator,
    RequestBuilder,
};
use anchor_lang::{prelude::Pubkey, AccountDeserialize, Discriminator};
use futures::stream::StreamExt;
use futures::Future;
use solana_account_decoder::UiAccountEncoding;
use solana_client::{
    nonblocking::{pubsub_client::PubsubClient, rpc_client::RpcClient},
    pubsub_client::PubsubClientError,
    rpc_config::{
        RpcAccountInfoConfig, RpcProgramAccountsConfig, RpcSendTransactionConfig,
        RpcTransactionLogsConfig, RpcTransactionLogsFilter,
    },
    rpc_filter::{Memcmp, RpcFilterType},
};
use solana_sdk::{
    commitment_config::CommitmentConfig, signature::Signature, signer::Signer,
    transaction::Transaction,
};
use std::{ops::Deref, pin::Pin, sync::Arc};
use tokio::{
    spawn,
    sync::mpsc::{unbounded_channel, UnboundedReceiver},
    task::JoinHandle,
};

type UnsubscribeFn = Box<dyn FnOnce() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send>;

pub struct EventUnsubscriber {
    handle: JoinHandle<Result<(), ClientError>>,
    rx: UnboundedReceiver<UnsubscribeFn>,
}

impl EventUnsubscriber {
    /// Unsubscribe gracefully.
    pub async fn unsubscribe(mut self) {
        if let Some(unsubscribe) = self.rx.recv().await {
            unsubscribe().await;
        }

        let _ = self.handle.await;
    }
}

impl<C: Deref<Target = impl Signer> + Clone> Program<C> {
    /// Returns the account at the given address
    pub async fn account<T: AccountDeserialize>(&self, address: Pubkey) -> Result<T, ClientError> {
        let rpc_client = RpcClient::new_with_commitment(
            self.cfg.cluster.url().to_string(),
            self.cfg.options.unwrap_or_default(),
        );
        let account = rpc_client
            .get_account_with_commitment(&address, CommitmentConfig::processed())
            .await?
            .value
            .ok_or(ClientError::AccountNotFound)?;
        let mut data: &[u8] = &account.data;
        T::try_deserialize(&mut data).map_err(Into::into)
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
        let account_type_filter =
            RpcFilterType::Memcmp(Memcmp::new_base58_encoded(0, &T::discriminator()));
        let config = RpcProgramAccountsConfig {
            filters: Some([vec![account_type_filter], filters].concat()),
            account_config: RpcAccountInfoConfig {
                encoding: Some(UiAccountEncoding::Base64),
                ..RpcAccountInfoConfig::default()
            },
            ..RpcProgramAccountsConfig::default()
        };
        Ok(ProgramAccountsIterator {
            inner: self
                .rpc()
                .get_program_accounts_with_config(&self.id(), config)
                .await?
                .into_iter()
                .map(|(key, account)| {
                    Ok((key, T::try_deserialize(&mut (&account.data as &[u8]))?))
                }),
        })
    }

    pub fn rpc(&self) -> RpcClient {
        RpcClient::new_with_commitment(
            self.cfg.cluster.url().to_string(),
            self.cfg.options.unwrap_or_default(),
        )
    }

    /// Subscribe to program logs.
    ///
    /// Returns an [`EventUnsubscriber`] to unsubscribe and close connection gracefully.
    pub async fn on<T: anchor_lang::Event + anchor_lang::AnchorDeserialize>(
        &self,
        f: impl Fn(&EventContext, T) + Send + 'static,
    ) -> Result<EventUnsubscriber, ClientError> {
        let client = Arc::new(PubsubClient::new(self.cfg.cluster.ws_url()).await?);
        let config = RpcTransactionLogsConfig {
            commitment: self.cfg.options,
        };

        let program_id_str = self.program_id.to_string();
        let filter = RpcTransactionLogsFilter::Mentions(vec![program_id_str.clone()]);

        let (tx, rx) = unbounded_channel::<_>();

        let handle = spawn(async move {
            let client = Arc::clone(&client);
            let (mut notifications, unsubscribe) = client.logs_subscribe(filter, config).await?;

            tx.send(unsubscribe).map_err(|e| {
                ClientError::SolanaClientPubsubError(PubsubClientError::UnexpectedMessageError(
                    e.to_string(),
                ))
            })?;

            while let Some(logs) = notifications.next().await {
                let ctx = EventContext {
                    signature: logs.value.signature.parse().unwrap(),
                    slot: logs.context.slot,
                };
                let events = parse_logs_response(logs, &program_id_str);
                for e in events {
                    f(&ctx, e);
                }
            }
            Ok::<(), ClientError>(())
        });

        Ok(EventUnsubscriber { handle, rx })
    }
}

impl<'a, C: Deref<Target = impl Signer> + Clone> RequestBuilder<'a, C> {
    pub async fn signed_transaction(&self) -> Result<Transaction, ClientError> {
        let latest_hash = RpcClient::new_with_commitment(self.cluster.to_owned(), self.options)
            .get_latest_blockhash()
            .await?;
        let tx = self.signed_transaction_with_blockhash(latest_hash)?;

        Ok(tx)
    }

    pub async fn send(self) -> Result<Signature, ClientError> {
        let rpc_client = RpcClient::new_with_commitment(self.cluster.to_owned(), self.options);
        let latest_hash = rpc_client.get_latest_blockhash().await?;
        let tx = self.signed_transaction_with_blockhash(latest_hash)?;

        rpc_client
            .send_and_confirm_transaction(&tx)
            .await
            .map_err(Into::into)
    }

    pub async fn send_with_spinner_and_config(
        self,
        config: RpcSendTransactionConfig,
    ) -> Result<Signature, ClientError> {
        let rpc_client = RpcClient::new_with_commitment(self.cluster.to_owned(), self.options);
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
}
