use alloy::hex;
use ark_bls12_381::{Bls12_381, Fr};
use ark_poly_commit::kzg10::Commitment;
use ark_serialize::CanonicalDeserialize;
use eyre::Result;
use std::io::{Cursor, Read};

pub fn hex_to_commitment(hex: &str) -> Result<Commitment<Bls12_381>, Box<dyn std::error::Error>> {
    let bytes = hex::decode(hex)?;
    let commitment = Commitment::<Bls12_381>::deserialize_compressed(&bytes[..])?;
    Ok(commitment)
}

// Convert bytes to field elements
pub fn bytes_to_field_elements(
    bytes: &[u8],
    field_size: usize,
) -> Result<Vec<Fr>, Box<dyn std::error::Error>> {
    let mut padded_bytes = bytes.to_vec();
    let remainder = bytes.len() % field_size;
    if remainder != 0 {
        let padding = field_size - remainder;
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
