use anyhow::{Context, Result};
use azure_storage::clients::StorageClient;
use azure_storage_blobs::prelude::{AsContainerClient, ContainerClient};
use slight_common::BasicState;
use slight_runtime_configs::get_from_state;
use tracing::log;

use crate::providers::azure;

/// This is the underlying struct behind the `AzBlob` variant of the `KvImplementor` enum.
///
/// It provides a property that pertains solely to the azblob implementation
/// of this capability:
///     - `container_client`
///
/// As per its' usage in `KvImplementor`, it must `derive` `Debug`, and `Clone`.
#[derive(Debug, Clone)]
pub struct AzBlobImplementor {
    container_client: ContainerClient,
}

impl AzBlobImplementor {
    pub async fn new(slight_state: &BasicState, name: &str) -> Self {
        let storage_account_name = get_from_state("AZURE_STORAGE_ACCOUNT", slight_state)
            .await
            .unwrap();
        let storage_account_key = get_from_state("AZURE_STORAGE_KEY", slight_state)
            .await
            .unwrap();

        let container_client =
            StorageClient::new_access_key(storage_account_name, storage_account_key)
                .container_client(name);
        Self { container_client }
    }

    pub async fn get(&self, key: &str) -> Result<Vec<u8>> {
        let blob_client = self.container_client.blob_client(key);
        let res = azure::get(blob_client)
            .await
            .with_context(|| format!("failed to get value for key {}", key))?;
        Ok(res)
    }

    pub async fn set(&self, key: &str, value: &[u8]) -> Result<()> {
        let blob_client = self.container_client.blob_client(key);
        let value = Vec::from(value);
        azure::set(blob_client, value)
            .await
            .with_context(|| format!("failed to set value for key '{}'", key))?;
        Ok(())
    }

    pub async fn keys(&self) -> Result<Vec<String>> {
        let blobs = azure::list_blobs(self.container_client.clone())
            .await
            .with_context(|| "failed to list blobs")?;
        log::debug!("found blobs: {:?}", blobs);
        let keys = blobs
            .iter()
            .map(|blob| blob.name.clone())
            .collect::<Vec<String>>();
        Ok(keys)
    }

    pub async fn delete(&self, key: &str) -> Result<()> {
        let blob_client = self.container_client.blob_client(key);
        azure::delete(blob_client)
            .await
            .with_context(|| "failed to delete key's value")?;
        Ok(())
    }
}
