mod encoding;
mod files;
mod hashing;
mod params;
mod sponge;

use ark_bls12_381::Fr;
use ark_ff::PrimeField;
use ark_std::rand::{rngs::StdRng, SeedableRng};
use eyre::Result;
use reed_solomon_erasure::galois_8::ReedSolomon;

use secret_sharing_and_dkg::common::Share;
use secret_sharing_and_dkg::common::Shares;
use secret_sharing_and_dkg::shamir_ss::deal_secret;

use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

const FILE_SIZE: usize = 131072; // 128 KiB
const CHUNK_SIZE: usize = 16384; // 16 KiB
const TOTAL_SHARDS: usize = 14; // Total number of shards
const THRESHOLD: usize = 8; // Minimum number of shards required to reconstruct
const BLOB_SIZE: usize = 4096; // Number of field elements in a blob
const BLS_MODULUS: &str =
    "52435875175126190479447740508185965837690552500527637822603658699938581184513";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Blob data from random blob on blobscan
    // versioned_hash: 0x019284f9748164a57b0b17aba67690ccd559c0b39a91351a967000c1f709575c (0x01 = version, rest is SHA256 hash of KZG)
    // commitment: 0xb7864de608d22d35ff0d31fd193cabbdd25c5218d58a178ea161ada3f5d8b64aac0abc11b64ed8de5c88c1844ed17352
    // proof: 0x96ca77996cf91de924d06a1f58e78fc2186ce49fa2c437471bc4aca21208e3edfc0980600696b018543b4cfdfd07207e
    // size: 128 KiB
    // Stored on google: https://storage.googleapis.com/blobscan-production/1/01/92/84/019284f9748164a57b0b17aba67690ccd559c0b39a91351a967000c1f709575c.txt
    // Step 1: Split the file into segments
    let blob_path = "./blob/019284f9748164a57b0b17aba67690ccd559c0b39a91351a967000c1f709575c.txt";

    // Read the blob from the file
    let mut blob_file = File::open(blob_path)?;
    let mut blob_data = Vec::new();
    blob_file.read_to_end(&mut blob_data)?;
    // get file segment
    let chunks = segment_blob(&blob_data);

    // Encode each chunk using Reed-Solomon
    let rs = ReedSolomon::new(THRESHOLD, TOTAL_SHARDS - THRESHOLD)
        .expect("Invalid parameters for Reed-Solomon");
    let encoded_chunks = encode_chunks(&rs, &chunks)?;

    // Store each chunk in the sharded storage system
    for (i, chunk) in encoded_chunks.iter().enumerate() {
        let file_path = format!("./storage_node/chunk_{}.bin", i);
        store_chunk(&file_path, chunk)?;
    }

    println!("Enocded Chunks length: {}", encoded_chunks.len());

    // Retrieve and reconstruct the data
    //let recovered_data = reconstruct_data("./storage_node/chunk_", TOTAL_SHARDS)?;
    //assert_eq!(blob_data, recovered_data);
    println!(
        "Recovered data matches original blob {}",
        blob_data.len(),
        //recovered_data.len()
    );

    Ok(())
}

// divides the blob into chunks of 16 KiB
fn segment_blob(blob: &[u8]) -> Vec<Vec<u8>> {
    blob.chunks(CHUNK_SIZE)
        .map(|chunk| chunk.to_vec())
        .collect()
}

fn store_chunk<P: AsRef<Path>>(path: P, chunk: &[u8]) -> io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(chunk)?;
    Ok(())
}

// Encode the chunks using Reed-Solomon
fn encode_chunks(
    rs: &ReedSolomon,
    chunks: &[Vec<u8>],
) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
    let mut shards: Vec<_> = chunks
        .iter()
        .map(|chunk| {
            let mut shard = chunk.clone();
            shard.resize(CHUNK_SIZE, 0);
            shard
        })
        .collect();

    // Extend the vector to include parity shards
    shards.resize(TOTAL_SHARDS, vec![0; CHUNK_SIZE]);
    rs.encode(&mut shards)?;

    Ok(shards)
}
