use crate::{
    parse_logs_response, ClientError, EventContext, EventHandle, Program, ProgramAccountsIterator,
    RequestBuilder,
};
use anchor_lang::{prelude::Pubkey, AccountDeserialize, Discriminator};
use solana_account_decoder::UiAccountEncoding;
use solana_client::{
    pubsub_client::PubsubClient,
    rpc_client::RpcClient,
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
use std::ops::Deref;

impl<C: Deref<Target = impl Signer> + Clone> Program<C> {
    /// Returns the account at the given address.
    pub fn account<T: AccountDeserialize>(&self, address: Pubkey) -> Result<T, ClientError> {
        let rpc_client = RpcClient::new_with_commitment(
            self.cfg.cluster.url().to_string(),
            self.cfg.options.unwrap_or_default(),
        );
        let account = rpc_client
            .get_account_with_commitment(&address, CommitmentConfig::processed())?
            .value
            .ok_or(ClientError::AccountNotFound)?;
        let mut data: &[u8] = &account.data;
        T::try_deserialize(&mut data).map_err(Into::into)
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
                .get_program_accounts_with_config(&self.id(), config)?
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

    pub fn on<T: anchor_lang::Event + anchor_lang::AnchorDeserialize>(
        &self,
        f: impl Fn(&EventContext, T) + Send + 'static,
    ) -> Result<EventHandle, ClientError> {
        let addresses = vec![self.program_id.to_string()];
        let filter = RpcTransactionLogsFilter::Mentions(addresses);
        let ws_url = self.cfg.cluster.ws_url().to_string();
        let cfg = RpcTransactionLogsConfig {
            commitment: self.cfg.options,
        };
        let self_program_str = self.program_id.to_string();
        let (client, receiver) = PubsubClient::logs_subscribe(&ws_url, filter, cfg)?;
        std::thread::spawn(move || {
            while let Ok(logs) = receiver.recv() {
                let ctx = EventContext {
                    signature: logs.value.signature.parse().unwrap(),
                    slot: logs.context.slot,
                };
                let events: Vec<T> = parse_logs_response(logs, &self_program_str);
                for e in events {
                    f(&ctx, e);
                }
            }
        });
        Ok(client)
    }
}

impl<'a, C: Deref<Target = impl Signer> + Clone> RequestBuilder<'a, C> {
    pub fn signed_transaction(&self) -> Result<Transaction, ClientError> {
        let latest_hash =
            RpcClient::new_with_commitment(&self.cluster, self.options).get_latest_blockhash()?;
        let tx = self.signed_transaction_with_blockhash(latest_hash)?;

        Ok(tx)
    }

    pub fn send(self) -> Result<Signature, ClientError> {
        let rpc_client = RpcClient::new_with_commitment(&self.cluster, self.options);
        let latest_hash = rpc_client.get_latest_blockhash()?;
        let tx = self.signed_transaction_with_blockhash(latest_hash)?;

        rpc_client
            .send_and_confirm_transaction(&tx)
            .map_err(Into::into)
    }

    pub fn send_with_spinner_and_config(
        self,
        config: RpcSendTransactionConfig,
    ) -> Result<Signature, ClientError> {
        let rpc_client = RpcClient::new_with_commitment(&self.cluster, self.options);
        let latest_hash = rpc_client.get_latest_blockhash()?;
        let tx = self.signed_transaction_with_blockhash(latest_hash)?;

        rpc_client
            .send_and_confirm_transaction_with_spinner_and_config(
                &tx,
                rpc_client.commitment(),
                config,
            )
            .map_err(Into::into)
    }
}
