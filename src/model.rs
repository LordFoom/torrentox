use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Info {
    name: Option<String>,
}
pub struct TorrentFile {
    announce: String,
    info: Info,
}
