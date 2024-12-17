use std::{fmt::Debug, sync::Arc};

use jsonrpsee::ws_client::WsClientBuilder;

use crate::{avail::sdk::RawAvailClient};

/// An implementation of the `DataAvailabilityClient` trait that interacts with the Avail network.
#[derive(Debug, Clone)]
pub struct AvailClient {
    api_url: String,
    sdk_client: Arc<RawAvailClient>,
}

impl AvailClient {
    pub async fn new(app_id: u32, api_url: String, seed: &str) -> anyhow::Result<Self> {
        let sdk_client =
            RawAvailClient::new(app_id, seed).await?;

        Ok(Self {
            api_url,
            sdk_client: Arc::new(sdk_client),
        })
    }

   pub async fn dispatch_blob(
        &self,
        _: u32, // batch_number
        data: Vec<u8>,
    ) -> anyhow::Result<String> {
        let client = WsClientBuilder::default()
            .build(self.api_url.as_str())
            .await
            ?;

        let extrinsic = self
            .sdk_client
            .build_extrinsic(&client, data)
            .await
            ?;

        let block_hash = self
            .sdk_client
            .submit_extrinsic(&client, extrinsic.as_str())
            .await
            ?;
        let tx_id = self
            .sdk_client
            .get_tx_id(&client, block_hash.as_str(), extrinsic.as_str())
            .await
            ?;

        Ok(format!("{}:{}", block_hash, tx_id))
    }
}
