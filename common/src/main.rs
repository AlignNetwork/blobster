use alloy::hex;
use ark_bls12_381::{Bls12_381, Fr};
use ark_poly::{
    univariate::DensePolynomial, EvaluationDomain, Evaluations, GeneralEvaluationDomain,
};
use ark_poly_commit::kzg10::{Commitment, Proof, KZG10};

use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_std::rand::Rng;
use reed_solomon_erasure::galois_8::ReedSolomon;

use eyre::Result;
use std::fs::File;
use std::io::{self, Cursor, Read, Write};

use std::path::Path;

const FILE_SIZE: usize = 262146; // 128 KiB
const FIELD_SIZE: usize = 32; // 32 bytes
const CHUNK_SIZE: usize = 512; // 16 KiB
const TOTAL_SHARDS: usize = 64; // Total number of shards
const THRESHOLD: usize = 32; // Minimum number of shards required to reconstruct
const BLOB_SIZE: usize = 262146; // Number of field elements in a blob
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
    let blob_size = blob_file.metadata()?.len();
    println!("Blob file size: {} bytes", blob_size);
    // Convert blob data to field elements
    let field_elements = bytes_to_field_elements(&blob_data)?;
    println!("Hello, world!");
    // Encode each chunk using Reed-Solomon
    let rs = ReedSolomon::new(THRESHOLD, TOTAL_SHARDS - THRESHOLD)
        .expect("Invalid parameters for Reed-Solomon");
    let master_copy = encode_chunks_rs(&rs, &field_elements[..])?;

    // Store each chunk in the sharded storage system
    for (i, chunk) in master_copy.iter().enumerate() {
        let file_path = format!("./storage_node/chunk_{}.bin", i);
        store_chunk(&file_path, chunk)?;
    }

    let stored_commitment = hex_to_commitment("b7864de608d22d35ff0d31fd193cabbdd25c5218d58a178ea161ada3f5d8b64aac0abc11b64ed8de5c88c1844ed17352")?;

    // Simulate missing shards by setting them to None
    let mut shards: Vec<Option<Vec<u8>>> = master_copy.iter().cloned().map(Some).collect();
    shards[0] = None;
    shards[4] = None;
    // Try to reconstruct missing shards
    rs.reconstruct(&mut shards)?;

    // Convert back to normal shard arrangement
    let result: Vec<Vec<u8>> = shards.into_iter().filter_map(|x| x).collect();

    assert!(rs.verify(&result).unwrap());
    assert_eq!(master_copy, result);

    /* / Prove and verify possession of a random element from the blob data
       let rng = &mut ark_std::rand::thread_rng();
       let evaluation_index = rng.gen_range(0..field_elements.len());
       let proof = generate_proof(&field_elements[..], evaluation_index)?;

       let valid = verify_proof(
           &stored_commitment,
           &proof,
           evaluation_index,
           field_elements[evaluation_index],
       )?;
       assert!(valid, "The proof of possession is invalid");
    */
    println!("Data encoded, stored, and verified successfully.");

    Ok(())
}

fn store_chunk<P: AsRef<Path>>(path: P, chunk: &[u8]) -> io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(chunk)?;
    Ok(())
}

// Convert bytes to field elements
fn bytes_to_field_elements(bytes: &[u8]) -> Result<Vec<Fr>, Box<dyn std::error::Error>> {
    let mut padded_bytes = bytes.to_vec();
    let remainder = bytes.len() % FIELD_SIZE;
    if remainder != 0 {
        let padding = FIELD_SIZE - remainder;
        padded_bytes.extend(vec![0u8; padding]);
    }

    let mut field_elements = Vec::new();
    let mut cursor = Cursor::new(padded_bytes);

    while cursor.position() < cursor.get_ref().len() as u64 {
        let mut buf = vec![0u8; 32];
        cursor.read_exact(&mut buf)?;
        let elem = Fr::deserialize_compressed(&buf[..])?;
        field_elements.push(elem);
    }

    Ok(field_elements)
}

// Encode the chunks using Reed-Solomon
fn encode_chunks_rs(
    rs: &ReedSolomon,
    field_elements: &[Fr],
) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
    let mut shards: Vec<Vec<u8>> = field_elements
        .chunks(CHUNK_SIZE)
        .map(|chunk| {
            let mut shard = chunk
                .iter()
                .flat_map(|&elem| {
                    let mut bytes = Vec::new();
                    elem.serialize_compressed(&mut bytes).unwrap();
                    bytes
                })
                .collect::<Vec<u8>>();
            shard.resize(CHUNK_SIZE * FIELD_SIZE, 0);
            shard
        })
        .collect();

    // Extend the vector to include parity shards
    shards.resize(TOTAL_SHARDS, vec![0; CHUNK_SIZE * FIELD_SIZE]);
    rs.encode(&mut shards)?;

    Ok(shards)
}

fn hex_to_commitment(hex: &str) -> Result<Commitment<Bls12_381>, Box<dyn std::error::Error>> {
    let bytes = hex::decode(hex)?;
    let commitment = Commitment::<Bls12_381>::deserialize_compressed(&bytes[..])?;
    Ok(commitment)
}
