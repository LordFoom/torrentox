use std::fmt::Display;

use serde_bencode::Error as BencodeError;
use thiserror::Error;

#[derive(Debug, thiserror::Error)]
pub enum TorrentParseError {
    #[error("Failed to decode the torrent file")]
    BenCode(#[from] BencodeError),
    #[error("Failed to read the torrent file")]
    CannotOpenFile(#[from] std::io::Error),
}

impl Display for TorrentParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
