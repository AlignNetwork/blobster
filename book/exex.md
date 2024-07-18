# ExEx

`Folder: remote`

Devmode: `cargo run --bin remote-exex --release -- node --dev`


The Reth ExEx is adapted from [Reth&apos;s Remote ExEx example](https://github.com/paradigmxyz/reth-exex-examples/tree/main/remote). 

Process:

1. Block is found w/ blob sidecar
2. Reed Solomon Encode blob data into chunks
3. Randomly send to one of three nodes (for testing purposes)

Roadmap:

1. Explore saving blob data commitments in a merkle tree so to easily verify when we implement zk proofs


## Holesky

To sync with Holesky

Holesky Reth (EL Node):

`export ETHERSCAN_API_KEY=<YOUR-KEY> && cargo run --bin remote-exex --release -- node --chain holesky --debug.etherscan --datadir /<YOUR_DIR>/holesky/reth/  --authrpc.jwtsecret /mnt/<YOUR_DIR/holesky/jwt.hex --http --http.api all`

Holesky Lighthouse (CL Node): 

`lighthouse bn --network holesky     --checkpoint-sync-url https://holesky.beaconstate.ethstaker.cc/     --execution-endpoint http://localhost:8551     --execution-jwt /mnt/<YOUR_DIR>/holesky/jwt.hex --datadir /mnt/<YOUR_DIR>/holesky/lighthouse/ --disable-deposit-contract-sync`

