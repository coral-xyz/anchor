use anchor_lang::{solana_program::pubkey::Pubkey, AccountDeserialize, Discriminator};
use std::ops::Deref;

#[cfg(target_arch = "wasm32")]
use {
    crate::EventUnsubscriber,
    solana_client_wasm::{
        solana_sdk::{commitment_config::CommitmentConfig, signature::Signer},
        utils::{
            rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
            rpc_filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType},
        },
        WasmClient,
    },
    solana_extra_wasm::account_decoder::UiAccountEncoding,
};

#[cfg(not(target_arch = "wasm32"))]
use {
    crate::UnsubscribeFn,
    futures::StreamExt,
    solana_account_decoder::UiAccountEncoding,
    solana_client::{
        nonblocking::{
            pubsub_client::{PubsubClient, PubsubClientError},
            rpc_client::RpcClient as AsyncRpcClient,
        },
        rpc_client::RpcClient,
        rpc_config::{
            RpcAccountInfoConfig, RpcProgramAccountsConfig, RpcTransactionLogsConfig,
            RpcTransactionLogsFilter,
        },
        rpc_filter::{Memcmp, RpcFilterType},
    },
    solana_sdk::{commitment_config::CommitmentConfig, signature::Signer},
    std::sync::Arc,
    tokio::{
        sync::{
            mpsc::{unbounded_channel, UnboundedReceiver},
            RwLock,
        },
        task::JoinHandle,
    },
};

use crate::{
    parse_logs_response, ClientError, Config, EventContext, ProgramAccountsIterator, RequestBuilder,
};

/// Program is the primary client handle to be used to build and send requests.
pub struct Program<C> {
    pub(crate) program_id: Pubkey,
    pub(crate) cfg: Config<C>,
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) sub_client: Arc<RwLock<Option<PubsubClient>>>,
    #[cfg(all(not(feature = "async"), not(target_arch = "wasm32")))]
    pub(crate) rt: tokio::runtime::Runtime,
}

impl<C: Deref<Target = impl Signer> + Clone> Program<C> {
    pub fn payer(&self) -> Pubkey {
        self.cfg.payer.pubkey()
    }

    #[cfg(any(feature = "async", not(target_arch = "wasm32")))]
    /// Returns a request builder.
    pub fn request(&self) -> RequestBuilder<C> {
        RequestBuilder::from(
            self.program_id,
            self.cfg.cluster.url(),
            self.cfg.payer.clone(),
            self.cfg.options,
            #[cfg(all(not(feature = "async"), not(target_arch = "wasm32")))]
            self.rt.handle(),
        )
    }

    pub fn id(&self) -> Pubkey {
        self.program_id
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn rpc(&self) -> RpcClient {
        RpcClient::new_with_commitment(
            self.cfg.cluster.url().to_string(),
            self.cfg.options.unwrap_or_default(),
        )
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn async_rpc(&self) -> AsyncRpcClient {
        AsyncRpcClient::new_with_commitment(
            self.cfg.cluster.url().to_string(),
            self.cfg.options.unwrap_or_default(),
        )
    }

    #[cfg(target_arch = "wasm32")]
    pub fn async_rpc(&self) -> WasmClient {
        WasmClient::new_with_commitment(
            self.cfg.cluster.url(),
            self.cfg.options.unwrap_or_default(),
        )
    }

    pub(crate) async fn account_internal<T: AccountDeserialize>(
        &self,
        address: Pubkey,
    ) -> Result<T, ClientError> {
        #[cfg(not(target_arch = "wasm32"))]
        let account = self
            .async_rpc()
            .get_account_with_commitment(&address, CommitmentConfig::processed())
            .await?
            .value
            .ok_or(ClientError::AccountNotFound)?;

        #[cfg(target_arch = "wasm32")]
        let account = self
            .async_rpc()
            .get_account_with_commitment(&address, CommitmentConfig::processed())
            .await?
            .ok_or(ClientError::AccountNotFound)?;

        let mut data: &[u8] = &account.data;
        T::try_deserialize(&mut data).map_err(Into::into)
    }

    pub(crate) async fn accounts_lazy_internal<T: AccountDeserialize + Discriminator>(
        &self,
        filters: Vec<RpcFilterType>,
    ) -> Result<ProgramAccountsIterator<T>, ClientError> {
        #[cfg(not(target_arch = "wasm32"))]
        let account_type_filter =
            RpcFilterType::Memcmp(Memcmp::new_base58_encoded(0, &T::discriminator()));
        #[cfg(target_arch = "wasm32")]
        let account_type_filter = RpcFilterType::Memcmp(Memcmp {
            offset: 0,
            bytes: MemcmpEncodedBytes::Base58(bs58::encode(&T::discriminator()).into_string()),
            encoding: None,
        });

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
                .async_rpc()
                .get_program_accounts_with_config(&self.id(), config)
                .await?
                .into_iter()
                .map(|(key, account)| {
                    Ok((key, T::try_deserialize(&mut (&account.data as &[u8]))?))
                }),
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    async fn init_sub_client_if_needed(&self) -> Result<(), ClientError> {
        let lock = &self.sub_client;
        let mut client = lock.write().await;

        if client.is_none() {
            let sub_client = PubsubClient::new(self.cfg.cluster.ws_url()).await?;
            *client = Some(sub_client);
        }

        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn on_internal<T: anchor_lang::Event + anchor_lang::AnchorDeserialize>(
        &self,
        f: impl Fn(&EventContext, T) + Send + 'static,
    ) -> EventUnsubscriber {
        let client = self.async_rpc();
        let config = solana_client_wasm::utils::rpc_config::RpcTransactionLogsConfig {
            commitment: self.cfg.options,
        };
        let program_id_str = self.program_id.to_string();
        let filter =
            solana_client_wasm::utils::rpc_config::RpcTransactionLogsFilter::Mentions(vec![
                program_id_str.clone(),
            ]);

        let id = client
            .logs_subscribe(filter, config, move |ctx| {
                let event_ctx = EventContext {
                    signature: ctx.value.signature.parse().unwrap(),
                    slot: ctx.context.slot,
                };
                let events = parse_logs_response(ctx, &program_id_str);
                for e in events {
                    f(&event_ctx, e);
                }
            })
            .await;

        EventUnsubscriber { id, client }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) async fn on_internal<T: anchor_lang::Event + anchor_lang::AnchorDeserialize>(
        &self,
        f: impl Fn(&EventContext, T) + Send + 'static,
    ) -> Result<
        (
            JoinHandle<Result<(), ClientError>>,
            UnboundedReceiver<UnsubscribeFn>,
        ),
        ClientError,
    > {
        self.init_sub_client_if_needed().await?;
        let (tx, rx) = unbounded_channel::<_>();
        let config = RpcTransactionLogsConfig {
            commitment: self.cfg.options,
        };
        let program_id_str = self.program_id.to_string();
        let filter = RpcTransactionLogsFilter::Mentions(vec![program_id_str.clone()]);

        let lock = Arc::clone(&self.sub_client);

        let handle = tokio::spawn(async move {
            if let Some(ref client) = *lock.read().await {
                let (mut notifications, unsubscribe) =
                    client.logs_subscribe(filter, config).await?;

                tx.send(unsubscribe).map_err(|e| {
                    ClientError::SolanaClientPubsubError(PubsubClientError::RequestFailed {
                        message: "Unsubscribe failed".to_string(),
                        reason: e.to_string(),
                    })
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
            }
            Ok::<(), ClientError>(())
        });

        Ok((handle, rx))
    }
}
