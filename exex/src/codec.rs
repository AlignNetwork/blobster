use crate::proto;

pub struct ExExNotification {
    pub node_id: u32,
    pub chunk_index: u32,
    pub chunk: Vec<u8>,
    pub name: String,
}

impl From<&ExExNotification> for proto::BlobChunk {
    fn from(notification: &ExExNotification) -> Self {
        proto::BlobChunk {
            node_id: notification.node_id,
            chunk_index: notification.chunk_index,
            chunk: notification.chunk.clone(),
            name: notification.name.clone(),
        }
    }
}

impl From<proto::BlobChunk> for ExExNotification {
    fn from(blob_chunk: proto::BlobChunk) -> Self {
        ExExNotification {
            node_id: blob_chunk.node_id,
            chunk_index: blob_chunk.chunk_index,
            chunk: blob_chunk.chunk,
            name: blob_chunk.name,
        }
    }
}
