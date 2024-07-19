# Blobster Book

_Documentation for Blobster_

[![Telegram Chat][tg-badge]][tg-url]

Blobster is a blazing fast and cheap **Ethereum Blob storage system.**

Currently, the system can be used locally with the Reth development settings and comes with a Consensus Layer client to send simulated blobs to the node. We will move to using Holesky once the node is done syncing (soon!).


<img src="https://raw.githubusercontent.com/alignnetwork/blobster/master/assets/blobster_banner.png" style="border-radius: 20px">

### Why?

The motivation behind creating a network to offer longer term storage solutions to blob storage is to build support and tooling to support the underexplored world of blobs and to test out Danksharding techniques in a real world setting. We think by taking advantage of the KZG commitment nature of blobs, we can streamline a SNARK proving system for storage nodes. Also taking advantage of erasure encoding and some mixing techniques, we believe we can achieve lower storage requirements than replication while still maintaining significant economic security. These ideas are a WIP and may change or be proven incorrect, however we think we have enough of a base to field outside opinions. 

### How?

We run an [Reth Node w/ an ExEx](https://github.com/paradigmxyz/reth) that detects blobs, queries the consensus layer, erasure encodes the blobs and stores them in a series of storage nodes erasure encoding for cheap and efficient long term storage of blobs.

Curious about blobs? Check this guide by [Ethereum](https://ethereum.org/en/roadmap/danksharding/) and the [EIP-4844](https://www.eip4844.com/) website.

### Roadmap

Currently the system can be run locally, and we are working to supporting bringing the system live on Holesky after we complete the following steps.

High level goals:

1. Finish syncing holesky (WIP)
2. Implement Retrieval of blobs
3. Create a SNARK proving system for storage nodes to prove they are storing data & accompanying Merkle Proof state root tracking
4. Host network to open up storage node solutions
5. Implement a Global Storage Node Register


[tg-badge]: https://img.shields.io/endpoint?color=neon&logo=telegram&label=chat&url=https%3A%2F%2Ftg.sumanjay.workers.dev%2Falign%5Fblobster
[tg-url]: https://t.me/align_blobster