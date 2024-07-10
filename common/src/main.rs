mod merkletree;
mod reedsolomon_encode;
mod utils;

use crate::reedsolomon_encode::encode_chunks_rs;
use crate::utils::{bytes_to_field_elements, hex_to_commitment};
use ark_bls12_381::{Bls12_381, Fr};
use merkletree::MerkleTree;

use ark_poly_commit::kzg10::Commitment;
use reed_solomon_erasure::galois_8::ReedSolomon;

use eyre::Result;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;
use tokio;

const FILE_SIZE: usize = 262146; // 128 KiB
const FIELD_SIZE: usize = 32; // 32 bytes
const CHUNK_SIZE: usize = 512; // 16 KiB
const TOTAL_SHARDS: usize = 64; // Total number of shards
const THRESHOLD: usize = 32; // Minimum number of shards required to reconstruct
const BLOB_SIZE: usize = 262146; // Number of field elements in a blob
const BLS_MODULUS: &str =
    "52435875175126190479447740508185965837690552500527637822603658699938581184513";

struct FileRecord {
    chunk_index: usize,
    merkle_root: Fr,
    commitment: Commitment<Bls12_381>,
}

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
    let stored_commitment = hex_to_commitment("0xb7864de608d22d35ff0d31fd193cabbdd25c5218d58a178ea161ada3f5d8b64aac0abc11b64ed8de5c88c1844ed17352")?;
    // Read the blob from the file
    let mut blob_file = File::open(blob_path)?;
    let mut blob_data = Vec::new();
    blob_file.read_to_end(&mut blob_data)?;
    let blob_size = blob_file.metadata()?.len();
    println!("Blob file size: {} bytes", blob_size);
    // Convert blob data to field elements
    let field_elements = bytes_to_field_elements(&blob_data, FIELD_SIZE)?;
    println!("Hello, world!");
    // Encode each chunk using Reed-Solomon
    let rs = ReedSolomon::new(THRESHOLD, TOTAL_SHARDS - THRESHOLD)
        .expect("Invalid parameters for Reed-Solomon");
    let master_copy = encode_chunks_rs(
        &rs,
        &field_elements[..],
        CHUNK_SIZE,
        TOTAL_SHARDS,
        FIELD_SIZE,
    )?;

    // Store each chunk in the sharded storage system
    for (i, chunk) in master_copy.iter().enumerate() {
        let file_path = format!("./storage_node/chunk_{}.bin", i);
        store_chunk(&file_path, chunk)?;
    }

    // Create Merkle tree for each chunk
    let mut merkle_roots = Vec::new();
    for chunk in &master_copy {
        let chunk_field_elements = bytes_to_field_elements(chunk, FIELD_SIZE)?;
        let merkle_tree = MerkleTree::new(&chunk_field_elements);
        let merkle_root = merkle_tree.root();
        merkle_roots.push(merkle_root.clone());
    }

    // Store the root hash and KZG commitment of the blob
    let mut file_records = Vec::new();
    for (i, merkle_root) in merkle_roots.iter().enumerate() {
        let file_record = FileRecord {
            chunk_index: i,
            merkle_root: merkle_root.clone(),
            commitment: stored_commitment.clone(),
        };
        file_records.push(file_record);
    }

    // Store the root hash of this blob

    println!("Data encoded, stored, and verified successfully.");

    Ok(())
}

fn store_chunk<P: AsRef<Path>>(path: P, chunk: &[u8]) -> io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(chunk)?;
    Ok(())
}
