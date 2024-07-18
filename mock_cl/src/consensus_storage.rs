use rusqlite::{params, Connection, Result};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct BlobConsensusRow {
    pub block_hash: String,
    pub kzg_commitment: String,
    pub blob_data: String,
    pub kzg_proof: String,
}

pub struct BlobConsensusStorage {
    path: std::path::PathBuf,
}

pub const DB_PATH: &str = "mock_cl/blobs.db";

pub fn get_db_path() -> &'static Path {
    Path::new(DB_PATH)
}

impl BlobConsensusStorage {
    pub fn new(db_path: &Path) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS blobs (
                block_hash TEXT PRIMARY KEY,
                commitment_hash TEXT,
                blob_data TEXT,
                kzg_proof TEXT
            )",
            [],
        )?;
        Ok(Self { path: db_path.to_path_buf() })
    }

    pub fn insert_blob(
        &self,
        block_hash: &str,
        commitment_hash: &str,
        blob_data: &str,
        kzg_proof: &str,
    ) -> Result<()> {
        let conn = Connection::open(&self.path)?;
        conn.execute(
            "INSERT OR REPLACE INTO blobs (block_hash, commitment_hash, blob_data, kzg_proof) VALUES (?,?, ?, ?)",
            params![block_hash,commitment_hash, blob_data, kzg_proof],
        )?;
        Ok(())
    }

    pub fn get_blob(&self, commitment_hash: &str) -> Result<Option<BlobConsensusRow>> {
        let conn = Connection::open(&self.path)?;
        let mut stmt = conn.prepare("SELECT * FROM blobs WHERE block_hash = ?")?;
        let mut rows = stmt.query(params![commitment_hash])?;

        if let Some(row) = rows.next()? {
            let blob_data_hex: String = row.get(2)?;
            Ok(Some(BlobConsensusRow {
                block_hash: row.get(0)?,
                kzg_commitment: row.get(1)?,
                blob_data: blob_data_hex,
                kzg_proof: row.get(3)?,
            }))
        } else {
            Ok(None)
        }
    }
    // command to get all blobs
    pub fn get_all_blobs(&self) -> Result<Vec<BlobConsensusRow>> {
        let conn = Connection::open(&self.path)?;
        let mut stmt = conn.prepare("SELECT * FROM blobs")?;
        let mut rows = stmt.query([])?;

        let mut blobs = Vec::new();
        while let Some(row) = rows.next()? {
            blobs.push(BlobConsensusRow {
                block_hash: row.get(0)?,
                kzg_commitment: row.get(1)?,
                blob_data: row.get(2)?,
                kzg_proof: row.get(3)?,
            });
        }
        Ok(blobs)
    }

    pub fn delete_all_blobs(&self) -> Result<()> {
        let conn = Connection::open(&self.path)?;
        conn.execute("DELETE FROM blobs", [])?;
        Ok(())
    }
}
