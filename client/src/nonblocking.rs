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
use std::{collections::BTreeMap, ops::Deref, pin::Pin, sync::Arc};
use tokio::{
    spawn,
    sync::mpsc::{unbounded_channel, UnboundedReceiver},
    task::JoinHandle,
};

type UnsubscribeFunc = Box<dyn FnOnce() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send>;
type Event = (
    JoinHandle<Result<(), ClientError>>,
    UnboundedReceiver<UnsubscribeFunc>,
);

pub struct EventHandler {
    client: Arc<PubsubClient>,
    program_id: Pubkey,
    config: RpcTransactionLogsConfig,
    events: BTreeMap<u8, Event>,
}

impl EventHandler {
    pub fn new(client: PubsubClient, program_id: Pubkey, config: RpcTransactionLogsConfig) -> Self {
        Self {
            client: Arc::new(client),
            program_id,
            config,
            events: BTreeMap::new(),
        }
    }
    pub fn subscribe<T: anchor_lang::Event + anchor_lang::AnchorDeserialize>(
        &mut self,
        f: impl Fn(&EventContext, T) + Send + 'static,
    ) -> Result<u8, ClientError> {
        let addresses = vec![self.program_id.to_string()];
        let filter = RpcTransactionLogsFilter::Mentions(addresses);
        let self_program_str = self.program_id.to_string();
        let config = self.config.clone();
        let (unsubscribe_sender, unsubscribe_receiver) = unbounded_channel::<_>();

        let handle = spawn({
            let client = Arc::clone(&self.client);

            async move {
                let (mut notifications, unsubscribe) =
                    client.logs_subscribe(filter, config).await?;

                unsubscribe_sender.send(unsubscribe).map_err(|e| {
                    ClientError::SolanaClientPubsubError(PubsubClientError::UnexpectedMessageError(
                        e.to_string(),
                    ))
                })?;

                while let Some(logs) = notifications.next().await {
                    let ctx = EventContext {
                        signature: logs.value.signature.parse().unwrap(),
                        slot: logs.context.slot,
                    };
                    let events = parse_logs_response(logs, &self_program_str);
                    for e in events {
                        f(&ctx, e);
                    }
                }
                Ok::<(), ClientError>(())
            }
        });
        let id = find_id(&self.events)?;
        self.events.insert(id, (handle, unsubscribe_receiver));

        Ok(id)
    }

    pub async fn unsubscribe(&mut self, id: u8) {
        if let Some(mut event) = self.events.remove(&id) {
            if let Some(unsubscribe) = event.1.recv().await {
                unsubscribe().await;
            }

            let _ = event.0.await;
        }
    }
}

impl<C: Deref<Target = impl Signer> + Clone> Program<C> {
    /// Returns the account at the given address.
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

    pub async fn create_event_handler(&self) -> Result<EventHandler, ClientError> {
        let ws_url = self.cfg.cluster.ws_url();
        let client = PubsubClient::new(ws_url).await?;
        let config = RpcTransactionLogsConfig {
            commitment: self.cfg.options,
        };

        Ok(EventHandler::new(client, self.program_id, config))
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

fn find_id<T>(map: &BTreeMap<u8, T>) -> Result<u8, ClientError> {
    for i in u8::MIN..u8::MAX {
        if !map.contains_key(&i) {
            return Ok(i);
        }
    }
    Err(ClientError::NoIdFound)
}
