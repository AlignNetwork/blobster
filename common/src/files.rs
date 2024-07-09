use crate::encoding;
use crate::hashing;

use std::io;
use std::{fs::File, io::Read};

use alloy::signers::{local::PrivateKeySigner, Signer};
use eyre::Result;

pub fn file_to_buffer(file_path: &str) -> io::Result<Vec<u8>> {
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

#[tokio::main]
async fn file_to_commitment() -> Result<()> {
    let file_path = "bitcoin.pdf";

    // Convert file to buffer
    let buffer = file_to_buffer(file_path)?;

    // Encode the buffer into vector of bytes
    let file_segment = encoding::encode_segments(&buffer);
    // Compute Poseidon2 hash over the entire buffer (file)
    let hash = hashing::compute_poseidon2_hash(&file_segment);
    // Print the hash in hexadecimal format
    println!("Poseidon2 Hash of file: {:x?}", hash);

    // compute the segments of the file
    let segments = encoding::encode_file(&buffer);
    println!("Segment length: {}", segments.len());
    println!(
        "Number of segments: {}",
        segments.len() / encoding::FIELD_SIZE
    );

    /* / create a merkle tree of the segments using Poseidon2 as the hasher
    let merkle_tree =
        MerkleTree::<[u8; 32], Poseidon2Hash<F>, VecStore<[u8; 32]>, U8>::from_data(&segments)
            .unwrap();
    println!("Merkle tree: {:?}", merkle_tree); */

    // sign the message and send to the sequencer
    let signer = PrivateKeySigner::random();
    // The message to sign.
    let message = hash.as_bytes();
    // Sign the message asynchronously with the signer.
    let signature = signer.sign_message(message).await?;

    println!(
        "Signature produced by {}: {:?}",
        signer.address(),
        signature
    );
    println!(
        "Signature recovered address: {}",
        signature.recover_address_from_msg(&message[..])?
    );
    Ok(())
}
