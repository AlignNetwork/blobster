use alloy::{
    consensus::{SidecarBuilder, SimpleCoder},
    hex,
    network::{EthereumWallet, TransactionBuilder},
    providers::{Provider, ProviderBuilder},
    rpc::types::TransactionRequest,
    signers::local::PrivateKeySigner,
};
use eyre::Result;

use mock_cl::consensus_storage::{get_db_path, BlobConsensusStorage};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::time::Duration;
use tokio::time::sleep;

const BLOB_SIZE: usize = 131073; // 262146 characters in hex = 131073 bytes

#[tokio::main]
async fn main() -> Result<()> {
    let rpc_url: reqwest::Url = "http://127.0.0.1:8545".parse()?;
    // Testnet account1: 0x14dC79964da2C08b23698B3D3cc7Ca32193d9955
    let signer: PrivateKeySigner =
        "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".parse()?;
    let wallet = EthereumWallet::from(signer);
    let provider =
        ProviderBuilder::new().with_recommended_fillers().wallet(wallet).on_http(rpc_url.clone());

    let alice = "0x14dC79964da2C08b23698B3D3cc7Ca32193d9955".parse().unwrap();
    let consensus_storage = BlobConsensusStorage::new(get_db_path())?;

    for _ in 0..1 {
        // randomly generate data for each sidecar
        let mut rng = StdRng::seed_from_u64(42);
        let mut builder = SidecarBuilder::<SimpleCoder>::new();
        /*let blob_data: Vec<u8> = (0..BLOB_SIZE).map(|_| rng.gen()).collect(); */
        // Pad the blob_data with zeros if it's less than BLOB_SIZE
        builder.ingest(b"this is bitcoin");
        let sidecar = builder.build()?;

        //let commitment_hash = hex::encode(sidecar.commitments[0]);
        let commitment_hash = format!("0x{}", hex::encode(sidecar.commitments[0]));
        let kzg_proof = format!("0x{}", hex::encode(sidecar.proofs[0]));
        let blob_data_hex = format!("0x{}", hex::encode(sidecar.blobs[0]));

        //let blob_data_hex = format!("0x{}", hex::encode(&blob_data));
        //let blob_data = builder.get_data();
        println!("Sidecar commitment hash: {}", commitment_hash);
        println!("Sidecar proof: {}", kzg_proof);

        let gas_price = provider.get_gas_price().await?;
        let eip1559_est = provider.estimate_eip1559_fees(None).await?;
        let tx = TransactionRequest::default()
            .with_to(alice)
            .with_max_fee_per_blob_gas(gas_price)
            .with_max_fee_per_gas(eip1559_est.max_fee_per_gas)
            .with_max_priority_fee_per_gas(eip1559_est.max_priority_fee_per_gas)
            .with_blob_sidecar(sidecar);

        // Send the transaction and wait for the broadcast.
        let pending_tx = provider.send_transaction(tx).await?;

        println!("Pending transaction... {}", pending_tx.tx_hash());
        // Wait for the transaction to be included and get the receipt.
        let receipt = pending_tx.get_receipt().await?;
        println!("Transaction included in block {}", receipt.block_hash.unwrap());

        // Insert the blob into the consensus storage
        // TODO: Remove when the consensus client is running
        consensus_storage.insert_blob(
            &receipt.block_hash.unwrap().to_string(),
            &commitment_hash,
            &blob_data_hex,
            &kzg_proof,
        )?;

        sleep(Duration::from_secs(1)).await; // Wait for a second before sending the next transaction.
    }

    Ok(())
}
