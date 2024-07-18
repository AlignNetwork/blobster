use std::{fs::File, io::BufReader};

// Adapted from: https://github.com/paradigmxyz/reth/blob/main/examples/beacon-api-sidecar-fetcher/src/mined_sidecar.rs
use alloy_rpc_types_beacon::sidecar::{BeaconBlobBundle, BlobData, SidecarIterator};
use eyre::Result;
use reqwest::{Error, StatusCode};
use reth::{
    primitives::{BlobTransaction, SealedBlockWithSenders, B256},
    transaction_pool::BlobStoreError,
};
use serde::{Deserialize, Serialize};
use serde_json::{from_reader, json};
use thiserror::Error;
use tokio::time::{sleep, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockMetadata {
    pub block_hash: B256,
    pub block_number: u64,
    pub gas_used: u64,
}

#[derive(Debug, Clone)]
pub struct MinedBlob {
    pub transaction: BlobTransaction,
    pub block_metadata: BlockMetadata,
}

#[derive(Debug, Clone)]
pub struct ReorgedBlob {
    pub transaction_hash: B256,
    pub block_metadata: BlockMetadata,
}

#[derive(Debug, Clone)]
pub enum BlobTransactionEvent {
    Mined(MinedBlob),
    Reorged(ReorgedBlob),
}

// my beacon blob bundle
pub struct MyBeaconBlobBundle {
    /// Vec of individual blob data
    pub data: Vec<BlobData>,
}

/// SideCarError Handles Errors from both EL and CL
#[derive(Debug, Error)]
pub enum SideCarError {
    #[error("Reqwest encountered an error: {0}")]
    ReqwestError(Error),

    #[error("Failed to fetch transactions from the blobstore: {0}")]
    TransactionPoolError(BlobStoreError),

    #[error("400: {0}")]
    InvalidBlockID(String),

    #[error("404: {0}")]
    BlockNotFound(String),

    #[error("500: {0}")]
    InternalError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Data parsing error: {0}")]
    DeserializationError(String),

    #[error("{0} Error: {1}")]
    UnknownError(u16, String),
}

/// Query the Beacon Layer for missing BlobTransactions
pub async fn fetch_blobs_for_block(
    block_root: B256,
    block: SealedBlockWithSenders,
    txs: Vec<(reth::primitives::TransactionSigned, usize)>,
) -> Result<Vec<BlobTransactionEvent>, SideCarError> {
    const INITIAL_DELAY: Duration = Duration::from_millis(500); // 0.5 second initial delay

    // Initial delay
    sleep(INITIAL_DELAY).await;
    let client = reqwest::Client::new();
    let sidecar_url = format!("http://127.0.0.1:4242/eth/v1/beacon/blob_sidecars/{}", block_root);
    println!("in fetch blobs {:?}", sidecar_url);
    let response = match client.get(sidecar_url).header("Accept", "application/json").send().await {
        Ok(response) => response,
        Err(err) => {
            eprintln!("Error fetching sidecar: {:?}", err);
            return Err(SideCarError::ReqwestError(err));
        }
    };

    if !response.status().is_success() {
        return match response.status() {
            StatusCode::BAD_REQUEST => {
                eprintln!("Invalid request to server.");
                Err(SideCarError::InvalidBlockID("Invalid request to server.".to_string()))
            }
            StatusCode::NOT_FOUND => {
                eprintln!("Requested block not found.");
                Err(SideCarError::BlockNotFound("Requested block not found.".to_string()))
            }
            StatusCode::INTERNAL_SERVER_ERROR => {
                eprintln!("Server encountered an error.");
                Err(SideCarError::InternalError("Server encountered an error.".to_string()))
            }
            status => {
                eprintln!("Unhandled HTTP status: {}", status);
                Err(SideCarError::UnknownError(
                    status.as_u16(),
                    "Unhandled HTTP status.".to_string(),
                ))
            }
        };
    }

    // read from file
    /* let file_path = "mock_cl/example_blob_sidecar.json";
    let blob_bundle: BeaconBlobBundle = match read_blobs_from_file(file_path) {
        Ok(blobs_bundle) => {
          // Use blobs_bundle here
            println!("Successfully read blobs from file");
            blobs_bundle
          }
          Err(e) => return Err(SideCarError::DeserializationError(e.to_string())),
        }; */
    let s = match response.text().await {
        Ok(s) => {
            println!("Raw JSON response: {}", s);
            s
        }
        Err(e) => return Err(SideCarError::NetworkError(e.to_string())),
    };
    let blob_bundle: BeaconBlobBundle = match serde_json::from_str(&s) {
        Ok(b) => {
            println!("Successfully deserialized BeaconBlobBundle");
            b
        }
        Err(e) => {
            eprintln!("Failed to deserialize BeaconBlobBundle: {:?}", e);
            return Err(SideCarError::DeserializationError(e.to_string()));
        }
    };

    let mut sidecar_iterator = SidecarIterator::new(blob_bundle);

    let sidecars: Vec<BlobTransactionEvent> = txs
        .iter()
        .filter_map(|(tx, blob_len)| {
            sidecar_iterator.next_sidecar(*blob_len).map(|sidecar| {
                println!(
                    "Processing tx with hash: {:?}, sidecar available: {}",
                    tx.hash(),
                    sidecar.blobs.len()
                );

                let transaction = BlobTransaction::try_from_signed(tx.clone(), sidecar)
                    .expect("should not fail to convert blob tx if it is already eip4844");
                let block_metadata = BlockMetadata {
                    block_hash: block.hash(),
                    block_number: block.number,
                    gas_used: block.gas_used,
                };
                BlobTransactionEvent::Mined(MinedBlob { transaction, block_metadata })
            })
        })
        .collect();
    //println!("CL Response: {:?}", block.block.body);
    Ok(sidecars)
}

// Read from a JSON file
fn read_blobs_from_file(file_path: &str) -> Result<BeaconBlobBundle, Box<dyn std::error::Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let blobs_bundle: BeaconBlobBundle = from_reader(reader)?;
    Ok(blobs_bundle)
}
