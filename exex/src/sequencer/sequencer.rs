use reed_solomon_erasure::galois_8::ReedSolomon;

use reth::primitives::BlobTransactionSidecar;

const SHARD_SIZE: usize = 1024; // B
const DATA_SHARDS: usize = 128; // Total number of shards
const THRESHOLD: usize = 32; // Minimum number of shards required to reconstruct
const BLOB_SIZE: usize = 131072; // Number of field elements in a blob

pub async fn process_blob_sidecar(
    blob_sidecar: BlobTransactionSidecar,
) -> eyre::Result<Vec<Vec<u8>>> {
    println!("Processing blob sidecar");
    let mut all_shards = Vec::new();
    for blob_in in blob_sidecar.blobs {
        let blob_data: Vec<u8> = blob_in.to_vec();

        assert_eq!(blob_data.len(), BLOB_SIZE, "KZG blob must be exactly 131072 bytes");

        let data_shards = DATA_SHARDS;
        let parity_shards = THRESHOLD;
        let total_shards = data_shards + parity_shards;

        let rs = ReedSolomon::new(data_shards, parity_shards)
            .expect("Failed to initialize Reed-Solomon");

        let shard_size = blob_data.len() / data_shards;
        assert_eq!(shard_size, SHARD_SIZE, "Shard size should be 1024 bytes");

        let mut shards: Vec<Vec<u8>> = vec![vec![0u8; shard_size]; total_shards];

        for (i, chunk) in blob_data.chunks(shard_size).enumerate().take(data_shards) {
            shards[i].copy_from_slice(chunk);
        }

        rs.encode(&mut shards).expect("Failed to encode");

        all_shards.extend(shards);
    }

    Ok(all_shards)
}
