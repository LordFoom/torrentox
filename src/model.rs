use serde_bytes::ByteBuf;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq)]
pub struct Torrent {
    pub torrent_file: TorrentFile,
    pub name: String,
    pub file_path: String,
    pub announce_url: Option<String>,
    pub raw_bytes: Vec<u8>,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct TorrentFile {
    pub announce: Option<String>,
    pub info: Info,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Info {
    pub name: Option<String>,
    #[serde(rename = "piece length")]
    pub piece_length: u64,
    #[serde(rename = "meta version")]
    pub meta_version: Option<i8>,
    ///Must have either files or length, but not both, and not neither
    #[serde(rename = "files")]
    ///Must have either files or length, but not both, and not neither
    pub possible_files: Option<Vec<File>>,
    #[serde(rename = "length")]
    pub possible_length: Option<u64>,
    pub pieces: ByteBuf,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct File {
    pub length: u64,
    pub possible_path: Option<Vec<String>>,
}

///Request to the announce url. Note it will be used in an Http GET request
pub struct TrackerAnnounceRequest {
    ///The 20 byte sha1 hash of the bencoded form of the info value from the metainfo file.
    ///Note that this is a substring of the metainfo file. Don't forget to URL-encode this.
    info_hash: Vec<u8>,
    peer_id: String,
    ip: Option<String>,
    port_number: u64,
}
