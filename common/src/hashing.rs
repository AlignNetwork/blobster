use crate::params::POSEIDON2_BN256_PARAMS;
use crate::sponge::Poseidon2Hash;

use ark_ff::PrimeField;
use num_bigint::BigUint;
use zkhash::fields::bn256::FpBN256;

pub type F = FpBN256;

fn buffer_to_field_elements(buffer: &[u8]) -> Vec<F> {
    buffer
        .chunks(F::MODULUS_BIT_SIZE as usize / 8)
        .map(|chunk| F::from_le_bytes_mod_order(chunk))
        .collect()
}

pub fn compute_poseidon2_hash(buffer: &[u8]) -> String {
    // Convert the buffer to field elements
    let acc = buffer_to_field_elements(buffer);
    let poseidon2_input_size = 3; // assuming the rate defines the input size
    println!("poseidon2_input_size: {:?}", poseidon2_input_size);
    // Initialize the Poseidon hasher
    let hash = Poseidon2Hash::hash(&POSEIDON2_BN256_PARAMS, acc.as_slice(), false);
    println!("Hash: {:x?}", hash);
    // Convert the result to bytes
    let hash_bytes: String = BigUint::from(hash.into_bigint()).to_str_radix(10);

    hash_bytes
}
