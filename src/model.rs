use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TorrentFile {
    pub announce: Option<String>,
    pub info: Info,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Info {
    pub name: Option<String>,
    #[serde(rename = "piece length")]
    pub piece_length: u64,
    #[serde(rename = "meta version")]
    pub meta_version: Option<i8>,
    ///Must have either files or length, but not both, and not neither
    #[serde(rename = "file")]
    ///Must have either files or length, but not both, and not neither
    pub possible_files: Option<Vec<File>>,
    #[serde(rename = "length")]
    pub possible_length: Option<u64>,
    pub pieces: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct File {
    pub length: u64,
    pub path: Vec<String>,
}
