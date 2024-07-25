use clap::Parser;
use exex::proto::remote_ex_ex_client::RemoteExExClient;
use reed_solomon_erasure::galois_8::ReedSolomon;
use reth_tracing::{tracing::info, RethTracer, Tracer};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use tokio::sync::mpsc;

const NUM_NODES: usize = 3;
const DATA_SHARDS: usize = 128;
const PARITY_SHARDS: usize = 32;
const TOTAL_SHARDS: usize = DATA_SHARDS + PARITY_SHARDS;
const SHARD_SIZE: usize = 1024; // 1 KiB

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Commitment hash for the data retrieval
    #[clap(short, long)]
    commitment_hash: String,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let _ = RethTracer::new().init()?;

    let args = Args::parse();
    let commitment_hash = &args.commitment_hash;

    let chunks = retrieve_chunks_from_nodes(commitment_hash).await?;
    let reconstructed_data = reconstruct_data(chunks)?;
    // Save reconstructed data
    let output_file = format!("reconstructed_data_{}.bin", commitment_hash);
    std::fs::write(&output_file, &reconstructed_data)?;
    info!("Reconstructed data saved to: {}", output_file);

    Ok(())
}

async fn retrieve_chunks_from_nodes(
    commitment_hash: &str,
) -> eyre::Result<HashMap<usize, Vec<u8>>> {
    let (tx, mut rx) = mpsc::channel(1000);

    for node_id in 1..=NUM_NODES {
        let tx = tx.clone();
        let commitment_hash = commitment_hash.to_string();
        tokio::spawn(async move {
            if let Err(e) = retrieve_chunks_from_node(node_id, &commitment_hash, tx).await {
                eprintln!("Error retrieving chunks from node {}: {:?}", node_id, e);
            }
        });
    }

    let mut all_chunks = HashMap::new();
    while let Some((index, chunk)) = rx.recv().await {
        all_chunks.insert(index, chunk);
    }

    Ok(all_chunks)
}

async fn retrieve_chunks_from_node(
    _node_id: usize,
    _commitment_hash: &str,
    _tx: mpsc::Sender<(usize, Vec<u8>)>,
) -> eyre::Result<()> {
    let _ = _node_id;
    let mut client = RemoteExExClient::connect("http://[::1]:50051").await?;

    /* let request = tonic::Request::new(RetrieveChunksRequest {
        node_id: node_id as u32,
        commitment_hash: commitment_hash.to_string(),
    }); */

    /* let mut response = client.retrieve_chunks(request).await?.into_inner();

    while let Some(chunk) = response.next().await {
        match chunk {
            Ok(blob_chunk) => {
                let _ = tx.send((blob_chunk.chunk_index as usize, blob_chunk.chunk)).await;
            }
            Err(e) => {
                eprintln!("Failed to retrieve chunk from node {}: {:?}", node_id, e);
            }
        }
    } */

    Ok(())
}

fn reconstruct_data(
    chunks: HashMap<usize, Vec<u8>>,
) -> Result<Vec<u8>, reed_solomon_erasure::Error> {
    let mut shard_data = vec![vec![0u8; SHARD_SIZE]; TOTAL_SHARDS];
    let mut shard_present = vec![false; TOTAL_SHARDS];

    for (&index, chunk) in &chunks {
        shard_data[index].copy_from_slice(chunk);
        shard_present[index] = true;
    }

    let rs = ReedSolomon::new(DATA_SHARDS, PARITY_SHARDS).unwrap();
    let mut shard_data_slices: Vec<(&mut [u8], bool)> = shard_data
        .iter_mut()
        .zip(shard_present.iter())
        .map(|(shard, &present)| (shard.as_mut_slice(), present))
        .collect();

    rs.reconstruct(&mut shard_data_slices)?;

    let mut reconstructed_data = Vec::new();
    for shard in shard_data.iter().take(DATA_SHARDS) {
        reconstructed_data.extend_from_slice(shard);
    }

    Ok(reconstructed_data)
}

// Function to display a hexdump of the first n bytes
fn display_hexdump(data: &[u8], n: usize) {
    for (i, &byte) in data.iter().take(n).enumerate() {
        if i % 16 == 0 {
            print!("{:04x}: ", i);
        }
        print!("{:02x} ", byte);
        if (i + 1) % 16 == 0 || i == n - 1 {
            println!();
        }
    }
}

// Function to save the entire hexdump to a file
fn save_hexdump(data: &[u8], filename: &str) -> std::io::Result<()> {
    let mut file = File::create(filename)?;
    for (i, &byte) in data.iter().enumerate() {
        if i % 32 == 0 && i != 0 {
            writeln!(file)?;
        }
        write!(file, "{:02x}", byte)?;
    }
    Ok(())
}
