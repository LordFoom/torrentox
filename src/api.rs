use std::collections::HashMap;

use crate::{model::TorrentFile, parser};
use anyhow::Result;

///The call to the announce url is an HTTP request
pub fn construct_query_map(torrent_file: &TorrentFile) -> Result<HashMap<String, String>> {
    //we construct a map of param = > value
    let query_params = HashMap::new();
    let info_hash = parser::parse_info_hash(&torrent_file.info)?.to_vec();
    let info_hash_str = info_hash.join("");
    let v = urlencoding::encode(&info_hash_str);
    query_params.insert("info_hash", v.to_string());
    Ok(query_params)
}
