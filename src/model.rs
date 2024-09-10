use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Info {
    name: Option<String>,
    piece_length: usize,
}
pub struct TorrentFile {
    announce: String,
    info: Info,
}
