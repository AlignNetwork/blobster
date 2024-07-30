use clap::Parser;

#[derive(Debug, Parser)]
pub struct Libp2pArgsExt {
    #[clap(long, default_value = "/ip4/0.0.0.0/tcp/8080")]
    pub listen_address: String,
}
