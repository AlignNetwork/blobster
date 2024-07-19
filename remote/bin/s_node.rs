use clap::Parser;
use remote::proto::{remote_ex_ex_client::RemoteExExClient, NodeOnlineRequest, SubscribeRequest};
use reth_tracing::{tracing::info, RethTracer, Tracer};
use std::path::PathBuf;
use tokio::time::{sleep, Duration};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser, default_value_t = 1)]
    node_id: u32,

    #[clap(short, long, value_parser, default_value = "storage_node")]
    storage_dir: PathBuf,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let _ = RethTracer::new().init()?;

    let args = Args::parse();
    println!("Args: {:?}", args);

    let mut client = RemoteExExClient::connect("http://[::1]:10000")
        .await?
        .max_encoding_message_size(usize::MAX)
        .max_decoding_message_size(usize::MAX);

    // Notify the gRPC network that the node is online
    let online_request = NodeOnlineRequest { node_id: args.node_id };
    client.notify_online(online_request).await?;

    let request = SubscribeRequest { node_id: args.node_id };

    let mut stream = client.subscribe(request).await?.into_inner();
    info!("Subscription to gRPC stream established");

    std::fs::create_dir_all(&args.storage_dir)?;
    info!("Created storage directory: {}", args.storage_dir.display());
    std::fs::create_dir_all(&args.storage_dir)?;

    loop {
        match stream.message().await {
            Ok(Some(blob_chunk)) => {
                if blob_chunk.node_id == args.node_id {
                    info!("Received blob chunk for node {}", args.node_id);
                    let file_name = args.storage_dir.join(format!(
                        "chunk_{}_{}_{}.bin",
                        blob_chunk.name, blob_chunk.node_id, blob_chunk.chunk_index
                    ));
                    if let Err(e) = std::fs::write(&file_name, blob_chunk.chunk) {
                        eprintln!(
                            "Failed to save chunk to file: {}. Error: {:?}",
                            file_name.display(),
                            e
                        );
                    } else {
                        info!("Saved chunk to file: {}", file_name.display());
                    }
                }
            }
            Ok(None) => {
                info!("No more messages in the stream");
            }
            Err(e) => {
                eprintln!("Error receiving message: {:?}", e);
                sleep(Duration::from_secs(1)).await; // Delay to avoid busy loop on error
            }
        }
        sleep(Duration::from_millis(100)).await; // Small delay to avoid busy loop
    }
}
