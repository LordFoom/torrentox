use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Info {
    pub name: Option<String>,
    #[serde(rename = "piece length")]
    pub piece_length: i64,
    #[serde(rename = "meta version")]
    pub meta_version: Option<i8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TorrentFile {
    pub announce: Option<String>,
    pub info: Info,
}
