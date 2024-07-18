# Quick Start


1. Clone the Repo

`git clone https://github.com/align_network/blobster`

2. Run Reth and the ExEx

`cargo run --bin remote-exex --release -- node --dev`

3. Run the Mock Consensus Layer

Ensure you have libsql installed
`sudo apt-get install libsqlite3-dev`

`cargo run --bin mock-cl --release`

4. Send 10 Random data blobs

`cargo run --bin update_blocks --release`

## Storage Nodes

Storage Nodes are currently setup to have a max of 3

1. Run a Storage node

`cargo run --release --bin storage-node -- --node-id=1 --storage-dir=storage/node1`


## Code

```bash
├── mock_cl
│   ├── bin
│   │   ├── mock_cl.rs
│   │   └── update_blocks.rs
│   ├── blobs.db
│   ├── Cargo.toml
│   ├── example_returned_blob.json
│   └── src
│       ├── consensus_storage.rs
│       └── lib.rs
├── remote
│   ├── bin
│   │   ├── exex.rs
│   │   ├── read.rs
│   │   └── s_node.rs
│   ├── build.rs
│   ├── Cargo.toml
│   ├── proto
│   │   ├── exex.proto
│   │   ├── exex.rs
│   │   └── mod.rs
│   └── src
│       ├── blobs.rs
│       ├── codec.rs
│       ├── example_blob_sidecar.json
│       ├── lib.rs
│       ├── sequencer
│       │   ├── sequencer.rs
│       │   └── utils.rs
│       └── sequencer.rs
```




