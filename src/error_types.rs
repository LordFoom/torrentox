use serde_bencode::Error as BencodeError;
use thiserror::Error;

#[derive(Debug, thiserror::Error)]
pub enum TorrentParseError {
    #[error("Failed to decode the torrent file")]
    BenCode(#[from] BencodeError),
    #[error("Failed to open the torrent file")]
    CannotOpenFile(#[from] std::io::Error),
}
