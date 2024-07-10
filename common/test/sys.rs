#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_main_functionality() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let blob_path = temp_dir.path().join("blob.txt");

        // Create a sample blob file
        let mut blob_file = File::create(&blob_path)?;
        let sample_data = vec![0u8; FILE_SIZE];
        blob_file.write_all(&sample_data)?;

        // Mock the commitment
        let stored_commitment = hex_to_commitment("b7864de608d22d35ff0d31fd193cabbdd25c5218d58a178ea161ada3f5d8b64aac0abc11b64ed8de5c88c1844ed17352")?;

        // Read the blob from the file
        let mut blob_file = File::open(&blob_path)?;
        let mut blob_data = Vec::new();
        blob_file.read_to_end(&mut blob_data)?;
        let blob_size = blob_file.metadata()?.len();
        assert_eq!(blob_size, FILE_SIZE as u64);

        // Convert blob data to field elements
        let field_elements = bytes_to_field_elements(&blob_data)?;

        // Encode each chunk using Reed-Solomon
        let rs = ReedSolomon::new(THRESHOLD, TOTAL_SHARDS - THRESHOLD)
            .expect("Invalid parameters for Reed-Solomon");
        let master_copy = encode_chunks_rs(&rs, &field_elements[..])?;

        // Store each chunk in the sharded storage system
        for (i, chunk) in master_copy.iter().enumerate() {
            let file_path = temp_dir.path().join(format!("chunk_{}.bin", i));
            store_chunk(&file_path, chunk)?;
        }

        // Create Merkle tree for each chunk
        let mut merkle_roots = Vec::new();
        for chunk in &master_copy {
            let chunk_field_elements = bytes_to_field_elements(chunk)?;
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

        // Verify the number of file records
        assert_eq!(file_records.len(), master_copy.len());

        Ok(())
    }
}
