use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use remote::blobs::{fetch_blobs_for_block, BlobTransactionEvent};
use remote::proto::{
    remote_ex_ex_server::{RemoteExEx, RemoteExExServer},
    BlobChunk, NodeOnlineRequest, NodeOnlineResponse, SubscribeRequest as ProtoSubscribeRequest,
};
use remote::sequencer::sequencer::process_blob_sidecar;
use reth_exex::{ExExContext, ExExEvent};
use reth_node_api::FullNodeComponents;
use reth_node_ethereum::EthereumNode;
use reth_tracing::tracing::info;
use tokio::sync::{broadcast, mpsc};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Request, Response, Status};

const NUM_NODES: usize = 3;

#[derive(Debug, Clone)]
pub enum ExExNotification {
    BlobChunk { node_id: u32, chunk_index: u32, chunk: Vec<u8>, name: String },
    NodeOnline { node_id: u32 },
}

#[derive(Debug)]
struct ExExService {
    notifications: broadcast::Sender<ExExNotification>,
    online_nodes: broadcast::Sender<u32>,
}

#[tonic::async_trait]
impl RemoteExEx for ExExService {
    type SubscribeStream = ReceiverStream<Result<BlobChunk, Status>>;

    async fn subscribe(
        &self,
        _request: Request<ProtoSubscribeRequest>,
    ) -> Result<Response<Self::SubscribeStream>, Status> {
        let (tx, rx) = mpsc::channel(1000);
        //println!("Subscribing to notifications...");
        let mut notifications = self.notifications.subscribe();
        //println!("Subscribed to notifications");
        tokio::spawn(async move {
            while let Ok(notification) = notifications.recv().await {
                match notification {
                    ExExNotification::BlobChunk {
                        node_id: chunk_node_id,
                        chunk_index,
                        chunk,
                        name,
                    } => {
                        //info!("Received blob chunk from notification");
                        info!(
                            "Sending chunk {} (name: {}) to node {}",
                            chunk_index, name, chunk_node_id
                        );

                        let blob_chunk =
                            BlobChunk { node_id: chunk_node_id, chunk_index, chunk, name };
                        if tx.send(Ok(blob_chunk)).await.is_err() {
                            eprintln!("Failed to send blob chunk to gRPC stream");
                            break;
                        } else {
                        }
                    }
                    ExExNotification::NodeOnline { node_id } => {
                        println!("Received notification: Node {} is online", node_id);
                    }
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn notify_online(
        &self,
        request: Request<NodeOnlineRequest>,
    ) -> Result<Response<NodeOnlineResponse>, Status> {
        let node_id = request.into_inner().node_id;
        info!("Node {} is online", node_id);

        let response = NodeOnlineResponse { message: format!("Node {} is online", node_id) };
        Ok(Response::new(response))
    }
}

async fn exex<Node: FullNodeComponents>(
    mut ctx: ExExContext<Node>,
    notifications: broadcast::Sender<ExExNotification>,
) -> eyre::Result<()> {
    let mut rng = StdRng::from_entropy();
    while let Some(notification) = ctx.notifications.recv().await {
        if let Some(committed_chain) = notification.committed_chain() {
            let events = committed_chain
                // Get all blocks and receipts
                .blocks_and_receipts()
                // Get all receipts
                .flat_map(|(block, receipts)| {
                    block
                        .body
                        .iter()
                        .zip(receipts.iter().flatten())
                        .map(move |receipt| (block.clone(), receipt.clone()))
                        .collect::<Vec<_>>()
                });
            for (block, _) in events {
                let txs: Vec<_> = block
                    .transactions()
                    .filter(|tx| tx.is_eip4844())
                    .map(|tx| (tx.clone(), tx.blob_versioned_hashes().unwrap().len()))
                    .collect();
                println!("Block Hash: {:?}", block.hash());
                match fetch_blobs_for_block(block.hash(), block, txs).await {
                    Ok(blob_transactions) => {
                        println!("Found {} blob transactions", blob_transactions.len());
                        for blob_transaction in blob_transactions {
                            match blob_transaction {
                                BlobTransactionEvent::Mined(mined) => {
                                    let commitment =
                                        mined.transaction.sidecar.commitments[0].clone();

                                    match process_blob_sidecar(mined.transaction.sidecar).await {
                                        Ok(chunks) => {
                                            // Distribute each chunk randomly
                                            for (chunk_index, chunk) in
                                                chunks.into_iter().enumerate()
                                            {
                                                let node_id = rng.gen_range(1..=NUM_NODES) as u32;
                                                let notification = ExExNotification::BlobChunk {
                                                    node_id,
                                                    chunk_index: chunk_index as u32,
                                                    chunk,
                                                    name: commitment.to_string(),
                                                };
                                                println!(
                                                    "Sending chunk {} to node {}",
                                                    chunk_index, node_id
                                                );
                                                if let Err(_e) = notifications.send(notification) {
                                                    eprintln!("Failed to send chunk");
                                                }
                                            }
                                        }
                                        Err(_e) => {
                                            eprintln!("Error processing blob sidecar:");
                                        }
                                    }
                                }
                                BlobTransactionEvent::Reorged(reorged) => {
                                    println!("Reorged blob transaction: {:?}", reorged);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error fetching blob transactions: {:?}", e);
                    }
                }
            }
            ctx.events.send(ExExEvent::FinishedHeight(committed_chain.tip().number))?;
        }
    }

    Ok(())
}

fn main() -> eyre::Result<()> {
    reth::cli::Cli::parse_args().run(|builder, _| async move {
        let notifications = broadcast::channel(1000).0;
        let online_nodes = broadcast::channel(1).0;

        let server = Server::builder()
            .add_service(RemoteExExServer::new(ExExService {
                notifications: notifications.clone(),
                online_nodes: online_nodes.clone(),
            }))
            .serve("[::1]:10000".parse().unwrap());

        let handle = builder
            .node(EthereumNode::default())
            .install_exex("Remote", |ctx| async move { Ok(exex(ctx, notifications)) })
            .launch()
            .await?;

        handle.node.task_executor.spawn_critical("gRPC server", async move {
            server.await.expect("gRPC server crashed")
        });

        handle.wait_for_node_exit().await
    })
}
