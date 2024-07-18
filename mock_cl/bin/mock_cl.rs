use alloy::hex;
use alloy::primitives::B256;

use mock_cl::consensus_storage::{get_db_path, BlobConsensusStorage};

use alloy_rpc_types_beacon::header::{BeaconBlockHeader, Header};
use rusqlite::Result;
use serde_json::json;

use std::sync::Arc;
use std::sync::Mutex;
use warp::http::StatusCode;
use warp::Filter;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BeaconBlobBundle {
    /// Vec of individual blob data
    pub data: Vec<BlobData>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlobData {
    /// Blob index
    pub index: String,
    /// Blob data
    pub blob: Box<String>,
    /// The blob's commitment
    pub kzg_commitment: String,
    /// The blob's proof
    pub kzg_proof: String,
    /// The block header containing the blob
    pub signed_block_header: Header,
    /// The blob's inclusion proofs
    pub kzg_commitment_inclusion_proof: Vec<B256>,
}

#[derive(Serialize, Deserialize)]
struct BlobSidecar {
    index: String,
    blob: String,
    kzg_commitment: String,
    kzg_proof: String,
    signed_block_header: SignedBeaconBlockHeader,
    kzg_commitment_inclusion_proof: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct SignedBeaconBlockHeader {
    message: BeaconBlockHeader,
    signature: String,
}

#[tokio::main]
async fn main() {
    let conn = Arc::new(Mutex::new(
        BlobConsensusStorage::new(get_db_path()).expect("Failed to create BlobConsensusStorage"),
    ));

    let blob_route = warp::path!("eth" / "v1" / "beacon" / "blob_sidecars" / String)
        .and(with_db(conn.clone()))
        .and_then(|param: String, storage: Arc<Mutex<BlobConsensusStorage>>| {
            serve_blob(param, storage)
        });

    let all_blobs_route = warp::path!("eth" / "v1" / "beacon" / "all_blobs")
        .and(with_db(conn.clone()))
        .and_then(serve_all_blobs);

    let delete_all_blobs_route = warp::path!("eth" / "v1" / "beacon" / "delete_all_blobs")
        .and(with_db(conn.clone()))
        .and_then(delete_all_blobs);

    let routes = blob_route.or(all_blobs_route).or(delete_all_blobs_route);

    warp::serve(routes).run(([127, 0, 0, 1], 4242)).await;
}

fn with_db(
    conn: Arc<Mutex<BlobConsensusStorage>>,
) -> impl Filter<Extract = (Arc<Mutex<BlobConsensusStorage>>,), Error = std::convert::Infallible> + Clone
{
    warp::any().map(move || conn.clone())
}

async fn serve_all_blobs(
    storage: Arc<Mutex<BlobConsensusStorage>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let storage = storage.lock().unwrap();
    match storage.get_all_blobs() {
        Ok(blobs) => {
            let blobs_json: Vec<serde_json::Value> = blobs
                .into_iter()
                .map(|blob| {
                    json!({
                        "block_hash": blob.block_hash,
                        "kzg_commitment": blob.kzg_commitment,
                        "blob_data": hex::encode(&blob.blob_data),
                        "kzg_proof": blob.kzg_proof,
                    })
                })
                .collect();
            Ok(warp::reply::json(&blobs_json))
        }
        Err(_) => Ok(warp::reply::json(&Vec::<serde_json::Value>::new())),
    }
}

async fn delete_all_blobs(
    storage: Arc<Mutex<BlobConsensusStorage>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let storage = storage.lock().unwrap();
    let _ = storage.delete_all_blobs();
    Ok(warp::reply::json(&true))
}

async fn serve_blob(
    param: String,
    storage: Arc<Mutex<BlobConsensusStorage>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    println!("Searching for blob sidecars for block: {}", param);
    let storage_result = storage.lock();
    let storage = match storage_result {
        Ok(guard) => guard,
        Err(poison_error) => {
            eprintln!("Mutex was poisoned. Recovering...");
            poison_error.into_inner()
        }
    };
    // Patrially ref from example_returned_blob.json
    match storage.get_blob(&param) {
        Ok(Some(row)) => {
            println!("Found blob for block: {}", param);
            let response = json!({
              "data": [{
                "index": "0",
                "blob": Box::new(row.blob_data.clone()),
                "kzg_commitment": row.kzg_commitment.clone(),
                "kzg_proof": row.kzg_proof.clone(),
                "signed_block_header": {
                  "message": {
                    "slot": "1409759".to_string(),
                    "proposer_index": "110239".to_string(),
                    "parent_root": "0x83c2e78d90e9d4031c0de0db5782143ac38e0e7f41ad98f8b97dff90a270e6df".to_string(),
                    "state_root": "0x11122c310a39307f2d3150f9f368599dd8c5771786479314ed527002f10e6548".to_string(),
                    "body_root": "0xe63dab4a3275db621ef3a3a34848d24049c61f6c0e93deaf6b179f0e8aee97b2".to_string(),
                  },
                  "signature": "0x953e3b23dcc50ca430e7c9456a053ceba5990b1ee542c1631aa96a3cd998ba5bf0df86d027288ddac07e4100a312b44506b4c4ad8c6e1c6ce27d39d76627e1f729c3622130c97def6c76f39d29cbf1b1fe81204ed821c89389d8b61d22530455".to_string()
                },
                "kzg_commitment_inclusion_proof": [
                "0x6f375622fe38528180b8bbce850131c5c287115fc0a19693a85073289b3aa1fe",
                "0x4a97acf7425809951e2dfa23af457d1591f91a4c072fb5ae07a6c38b6ac02270",
                "0xcdfe025837f134df085d20c9f4f48ba7469b6fe66dfd3ffe68086e3331f2ff3c",
                "0xc78009fdf07fc56a11f122370658a353aaa542ed63e44c4bc15ff4cd105ab33c",
                "0x536d98837f2dd165a55d5eeae91485954472d56f246df256bf3cae19352a123c",
                "0x9efde052aa15429fae05bad4d0b1d7c64da64d03d7a1854a588c2cb8430c0d30",
                "0xd88ddfeed400a8755596b21942c1497e114c302e6118290f91e6772976041fa1",
                "0x87eb0ddba57e35f6d286673802a4af5975e22506c7cf4c64bb6be5ee11527f2c",
                "0x26846476fd5fc54a5d43385167c95144f2643f533cc85bb9d16b782f8d7db193",
                "0x506d86582d252405b840018792cad2bf1259f1ef5aa5f887e13cb2f0094f51e1",
                "0xffff0ad7e659772f9534c195c815efc4014ef1e1daed4404c06385d11192e92b",
                "0x6cf04127db05441cd833107a52be852868890e4317e6a02ab47683aa75964220",
                "0x0600000000000000000000000000000000000000000000000000000000000000",
                "0x792930bbd5baac43bcc798ee49aa8185ef76bb3b44ba62b91d86ae569e4bb535",
                "0x527b1cda425c4bf8128c2ebcd1c9d9f4d507237067a59ce5815079553b36c6f7",
                "0xdb56114e00fdd4c1f85c892bf35ac9a89289aaecb1ebd0a96cde606a748b5d71",
                "0xe01c0837cb2d1b2dcb110929f0f3922b07d6712ae1e7ba65fda1eed7d79de4af"
                ]

              }]

            });

            Ok(warp::reply::with_status(warp::reply::json(&response), StatusCode::OK))
        }
        Ok(None) => {
            println!("Blob not found for block: {}", param);
            Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({"error": "Blob not found"})),
                StatusCode::NOT_FOUND,
            ))
        }
        Err(e) => {
            eprintln!("Error fetching blob: {:?}", e);
            Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({"error": "Internal server error"})),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}
