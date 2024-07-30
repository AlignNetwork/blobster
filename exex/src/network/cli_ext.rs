// In network/cli_ext.rs
use clap::Args;
use libp2p::Multiaddr;

#[derive(Debug, Args)]
pub struct LibP2PArgsExt {
    #[clap(long)]
    pub listen_address: Option<Multiaddr>,

    #[clap(long)]
    pub peer: Option<Multiaddr>,
}
