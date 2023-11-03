#[cfg(target_arch = "wasm32")]
use solana_client_wasm::{solana_sdk::signature::Signature, WasmClient};

#[cfg(not(target_arch = "wasm32"))]
use {
    crate::{ClientError, UnsubscribeFn},
    solana_sdk::signature::Signature,
    std::marker::PhantomData,
    tokio::{runtime::Handle, sync::mpsc::UnboundedReceiver, task::JoinHandle},
};

#[derive(Debug)]
pub struct EventContext {
    pub signature: Signature,
    pub slot: u64,
}

#[cfg(not(target_arch = "wasm32"))]
pub struct EventUnsubscriber<'a> {
    pub(crate) handle: JoinHandle<Result<(), ClientError>>,
    pub(crate) rx: UnboundedReceiver<UnsubscribeFn>,
    #[cfg(not(feature = "async"))]
    pub(crate) runtime_handle: &'a Handle,
    pub(crate) _lifetime_marker: PhantomData<&'a Handle>,
}

#[cfg(not(target_arch = "wasm32"))]
impl<'a> EventUnsubscriber<'a> {
    pub(crate) async fn unsubscribe_internal(mut self) {
        if let Some(unsubscribe) = self.rx.recv().await {
            unsubscribe().await;
        }

        let _ = self.handle.await;
    }
}

#[cfg(target_arch = "wasm32")]
pub struct EventUnsubscriber {
    pub(crate) id: u64,
    pub(crate) client: WasmClient,
}

#[cfg(target_arch = "wasm32")]
impl EventUnsubscriber {
    pub(crate) async fn unsubscribe_internal(&self) {
        self.client.logs_unsubscribe(self.id).await;
    }
}
