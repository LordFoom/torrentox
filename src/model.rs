use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Info {
    pub name: Option<String>,
    pub piece_length: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TorrentFile {
    pub announce: String,
    pub info: Info,
}
