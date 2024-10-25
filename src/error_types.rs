use serde_bencode::Error as BencodeError;

#[derive(Debug, thiserror::Error)]
pub enum TorrentParseError {
    #[error("Failed to decode the torrent file")]
    BenCode(#[from] BencodeError),
    #[error("Failed to read the torrent file")]
    CannotOpenFile(#[from] std::io::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Deserializetion error: {0}")]
    Deserializetion(#[from] serde_bencode::Error),
}
