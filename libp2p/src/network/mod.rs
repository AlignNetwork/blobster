use futures::prelude::*;
use libp2p::{
    identity, kad, noise,
    swarm::{Swarm, SwarmEvent},
    tcp, yamux,
};
use libp2p_swarm_derive::NetworkBehaviour;
use std::time::Duration;
use tokio::sync::mpsc;
pub mod cli_ext;

#[derive(Debug)]
pub enum Event {
    Kademlia(kad::Event),
    Ping(libp2p::ping::Event),
}

#[derive(NetworkBehaviour)]
struct Behaviour {
    kademlia: kad::Behaviour<kad::store::MemoryStore>,
    ping: libp2p::ping::Behaviour,
}

pub struct Libp2pExEx {
    swarm: Swarm<Behaviour>,
    event_sender: mpsc::Sender<Event>,
}

impl Libp2pExEx {
    pub async fn new(listen_address: String) -> eyre::Result<Self> {
        let id_keys = identity::Keypair::generate_ed25519();
        let peer_id = id_keys.public().to_peer_id();

        let mut swarm = libp2p::SwarmBuilder::with_existing_identity(id_keys)
            .with_tokio()
            .with_tcp(tcp::Config::default(), noise::Config::new, yamux::Config::default)?
            .with_behaviour(|key| Behaviour {
                kademlia: kad::Behaviour::new(
                    peer_id,
                    kad::store::MemoryStore::new(key.public().to_peer_id()),
                ),
                ping: libp2p::ping::Behaviour::new(libp2p::ping::Config::new()),
            })?
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
            .build();

        swarm.behaviour_mut().kademlia.set_mode(Some(kad::Mode::Server));

        // Start listening on the provided address
        swarm.listen_on(listen_address.parse()?)?;
        let (event_sender, _event_receiver) = mpsc::channel(100);
        Ok(Self { swarm, event_sender })
    }

    pub async fn run(&mut self) -> eyre::Result<()> {
        loop {
            match self.swarm.select_next_some().await {
                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("Listening on {:?}", address);
                }
                SwarmEvent::Behaviour(event) => {}
                _ => {}
            }
        }
    }
}
