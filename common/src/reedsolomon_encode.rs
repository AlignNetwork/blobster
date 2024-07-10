use ark_bls12_381::Fr;
use ark_serialize::CanonicalSerialize;
use eyre::Result;
use reed_solomon_erasure::galois_8::ReedSolomon;

// Encode the chunks using Reed-Solomon
pub fn encode_chunks_rs(
    rs: &ReedSolomon,
    field_elements: &[Fr],
    chunk_size: usize,
    total_shards: usize,
    field_size: usize,
) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
    let mut shards: Vec<Vec<u8>> = field_elements
        .chunks(chunk_size)
        .map(|chunk| {
            let mut shard = chunk
                .iter()
                .flat_map(|&elem| {
                    let mut bytes = Vec::new();
                    elem.serialize_compressed(&mut bytes).unwrap();
                    bytes
                })
                .collect::<Vec<u8>>();
            shard.resize(chunk_size * field_size, 0);
            shard
        })
        .collect();

    // Extend the vector to include parity shards
    shards.resize(total_shards, vec![0; chunk_size * field_size]);
    rs.encode(&mut shards)?;

    Ok(shards)
}
