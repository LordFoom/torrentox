use std::collections::HashMap;

use crate::{model::TorrentFile, parser};
use anyhow::Result;

///The call to the announce url is an HTTP request
pub fn construct_query_map(torrent_file: &TorrentFile) -> Result<Hashmap<String, String>> {
    //we construct a map of param = > value
    let query_params = HashMap::new();
    let info_hash = parser::parse_info_hash(&torrent_file.info)?;
    query_params.insert("info_hash", String::from_utf8(info_hash));
    Ok(query_params)
}
