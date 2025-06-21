use bitvec::prelude::*;
use bitvec::vec::BitVec;
use serde::de::Error as DeError;
use serde::Deserialize as Serdedeserialize;
use serde_bencode::value::Value;
use serde_bytes::ByteBuf;
use serde_derive::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::{collections::BTreeMap, fmt::Display};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Torrent {
    pub torrent_file: TorrentFile,
    pub name: String,
    pub file_path: String,
    pub announce_url: Option<String>,
    ///The torrent file, as bytes, in case we want to get currently unmodeled fields later if
    ///needed
    pub raw_bytes: Vec<u8>,
    ///left == size-downloaded
    pub size: u64,
    pub downloaded: u64,
    pub uploaded: u64,
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct TorrentFile {
    ///Tracker url, right?
    pub announce: Option<String>,
    pub piece_length: i64,
    //pub piece_length: Option<i64>,
    pub info: Info,
    ///The raw value of the info dictionary, for turning into a info_hash
    pub info_hash: InfoHash,
}

impl<'de> Serdedeserialize<'de> for TorrentFile {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut map = BTreeMap::<String, Value>::deserialize(deserializer)?;
        let announce_url = map
            .remove("announce")
            //.ok_or_else(|| serde::de::Error::missing_field("announce"))?;
            .ok_or_else(|| D::Error::missing_field("announce"))?;

        let announce = match announce_url {
            Value::Bytes(bytes) => {
                String::from_utf8(bytes).map_err(|e| D::Error::custom(e.to_string()))?
            }
            _ => {
                return Err(D::Error::custom(
                    "Expected bencoded dictionary that contains 'announce'",
                ))
            }
        };

        let info_value = map
            .remove("info")
            .ok_or_else(|| DeError::missing_field("info"))?;

        let info_raw_bytes = match info_value {
            Value::Dict(_) => serde_bencode::to_bytes::<Value>(&info_value).map_err(|e| {
                let msg = format!("unbencoding failed, but why? {:?}", e);
                D::Error::custom(&msg)
            }),

            _ => return Err(D::Error::custom("Unbencoding it did not work")),
        }?;

        let mut info_hash = [0; 20];
        let hash_result = Sha1::digest(&info_raw_bytes);
        info_hash.copy_from_slice(&hash_result);

        let info: Info = serde_bencode::from_bytes(&info_raw_bytes)
            .map_err(|e| D::Error::custom(format!("Unbencoding into Info failed {}", e)))?;

        Ok(TorrentFile {
            announce: Some(announce),
            piece_length: info.piece_length as i64,
            info,
            info_hash,
        })
    }
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
    #[serde(deserialize_with = "deserialize_peer")]
    pub peers: Vec<Peer>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PeerResponse {}

fn deserialize_peer<'de, D>(deserializer: D) -> Result<Vec<Peer>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let buf = ByteBuf::deserialize(deserializer)?;
    let bytes = buf.into_vec();

    if bytes.len() % 6 != 0 {
        return Err(D::Error::custom(
            "Wrong byte length, this is a packed bit format so 6 bit units",
        ));
    }
    let mut peers = Vec::new();
    for chunk in bytes.chunks(6) {
        let ip = format!("{}.{}.{}.{}", chunk[0], chunk[1], chunk[2], chunk[3]);
        let port = u16::from_be_bytes([chunk[4], chunk[5]]);
        let peer = Peer { ip, port };
        peers.push(peer);
    }
    Ok(peers)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Peer {
    // pub id: String,
    pub ip: String,
    pub port: u16,
}

impl Display for Peer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.ip, self.port)
    }
}

pub type InfoHash = [u8; 20];
pub type PeerId = [u8; 20];
// pub type Handshake = [u8; 68];

pub struct TorrentSession {
    pub torrent: Torrent,
    ///The torrentox peer_id
    pub peer_id: PeerId,
    pub peers: Vec<Peer>,
}

///Handshake returned from the peer
#[derive(Debug)]
pub struct PeerHandshake {
    pub info_hash: InfoHash,
    pub peer_id: PeerId,
}

///State of our interaction with the peer
pub struct PeerState {
    pub is_choked: bool,
    pub is_interested: bool,
    pub peer_bitfield: BitVec<u8, Msb0>,
    pub num_pieces: usize,
}

impl PeerState {
    pub fn new(num_pieces: usize) -> Self {
        Self {
            is_choked: true,
            is_interested: false,
            num_pieces,
            peer_bitfield: bitvec![u8, Msb0; 0; num_pieces],
        }
    }

    pub fn has_piece(&self, index: usize) -> bool {
        self.peer_bitfield.get(index).map(|b| *b).unwrap_or(false)
    }

    pub fn update_have(&mut self, index: usize) {
        if index < self.num_pieces {
            self.peer_bitfield.set(index, true);
        }
    }

    pub fn update_bitfield(&mut self, data: &[u8]) {
        let mut bits = bitvec![u8, Msb0; 0; self.num_pieces];

        for (i, byte) in data.iter().enumerate() {
            for j in 0..8 {
                let bit_index = i * 8 + j;
                if bit_index >= self.num_pieces {
                    break;
                }
                let bit = (byte >> (7 - j)) & 1 != 0;
                bits.set(bit_index, bit);
            }
        }

        self.peer_bitfield = bits;
    }
}
