# Mock Consensus Layer

`Folder: mock_cl`

Ensure you have libsql installed `sudo apt-get install libsqlite3-dev`

Endpoints: 

`/eth/v1/beacon/blob_sidecars/<block header>` - block header that has a tx w/ a bob

`/eth/v1/beacon/all_blobs` - list all the blobs 

`/etc/v1/beacon/delete_all_blobs` - clears out db




To Run: 
`cargo run --bin mock-cl --release`

This crate is tasked with mimicing a Consensus Layer to aid in development. It is very bare bones but should allow you to test with an Reth ExEx in dev mode i.e. without having to sync the full node.
The Update Blocks saves files to a sqlite db and when queried, this server responds with the blob data, commitment and proof. Requires one blob per tx. Reccommended to follow the `update_blocks` function to mimic sending.