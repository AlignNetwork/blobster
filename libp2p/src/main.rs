use clap::Parser;

use exex::ExEx;
use network::{cli_ext::Libp2pArgsExt, Libp2pExEx};
use reth_node_ethereum::EthereumNode;
use reth_tracing::tracing::info;

mod exex;
mod network;

fn main() -> eyre::Result<()> {
    reth::cli::Cli::<Libp2pArgsExt>::parse().run(|builder, args| async move {
        let listen_address = args.listen_address;

        let handle = builder
            .node(EthereumNode::default())
            .install_exex("exex-libp2p", move |ctx| async move {
                // start Libp2p task
                let libp2p = Libp2pExEx::new(listen_address).await?;
                info!("Libp2p task started");
                // start exex task with libp2p
                Ok(ExEx::new(ctx, libp2p))
            })
            .launch()
            .await?;

        handle.wait_for_node_exit().await
    })
}
