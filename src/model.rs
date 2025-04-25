use color_eyre::Result;
use serde_bencode::value::Value;
use serde_bytes::{ByteBuf, Deserialize};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Torrent {
    pub torrent_file: TorrentFile,
    pub name: String,
    pub file_path: String,
    pub announce_url: Option<String>,
    pub raw_bytes: Vec<u8>,
    ///left == size-downloaded
    pub size: u64,
    pub downloaded: u64,
    pub uploaded: u64,
}

//impl Torrent {
//    pub fn decompose_info_into_values(&mut self) -> Result<()> {
//        let piece_length = if let Value::Dict(val_map) = self.torrent_file.info.clone() {
//            match val_map.get(b"piece length" as &[u8]) {
//                Some(Value::Int(length)) => Some(length),
//                _ => None,
//            }
//        } else {
//            None
//        };
//        Ok(())
//    }
//}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct TorrentFile {
    ///Tracker url, right?
    pub announce: Option<String>,
    pub piece_length: Option<i64>,
    pub info: Info,
    //pub info: Value,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
#[serde(untagged)]
pub enum TorrentFileInfo {
    SingleFile { length: usize },
    MultipleFiles { files: Vec<FileInfo> },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct FileInfo {
    ///File size in bytes
    pub length: usize,
    ///Paths for each file,split by directories
    pub path: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Info {
    pub name: Option<String>,
    #[serde(rename = "piece length")]
    pub piece_length: u64,
    #[serde(rename = "meta version")]
    pub meta_version: Option<i8>,
    ///Must have either files or length, but not both, and not neither
    #[serde(flatten)]
    pub file: TorrentFileInfo,
    /////Must have either files or length, but not both, and not neither
    //pub possible_files: Option<Vec<File>>,
    //#[serde(rename = "length")]
    //pub possible_length: Option<u64>,
    //pub pieces: ByteBuf,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct File {
    pub length: u64,
    pub possible_path: Option<Vec<String>>,
}

///Request to the announce url. Note it will be used in an Http GET request
///Question do we need to implement serde for this as we are using this for http...
#[derive(Serialize, Deserialize)]
pub struct TrackerAnnounceRequest {
    ///The 20 byte sha1 hash of the bencoded form of the info value from the metainfo file.
    ///Note that this is a substring of the metainfo file. Don't forget to URL-encode this.
    info_hash: Vec<u8>,
    peer_id: String,
    ip: Option<String>,
    port_number: u64,
    uploaded: u128,
    downloaded: u128,
    left: u128,
    event: Option<String>,
    numwant: Option<u32>,
}

#[derive(Serialize, Deserialize)]
pub struct TrackerAnnounceResponse {
    ///Number of seconds the downloader should wait between regular rerequests.
    pub interval: usize,
    pub peers: Vec<Peer>,
}

#[derive(Serialize, Deserialize)]
pub struct Peer {
    id: String,
    ip: String,
    port: u64,
}

pub type InfoHash = [u8; 20];
