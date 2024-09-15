use serde_bencode::Error as BencodeError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TorrentParseError {
    BenCode(#[from] BencodeError),
}
