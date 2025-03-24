use std::{collections::HashMap, str};

use crate::{model::TorrentFile, parser};
use anyhow::Result;

///The call to the announce url is an HTTP request
pub fn construct_query_map(torrent_file: &TorrentFile) -> Result<HashMap<String, String>> {
    //we construct a map of param = > value
    let mut query_params = HashMap::new();
    let info_hash = parser::parse_info_hash(&torrent_file.info)?;
    let info_hash_str = str::from_utf8(&info_hash)?;
    let v = urlencoding::encode(info_hash_str);
    query_params.insert("info_hash".to_string(), v.to_string());
    Ok(query_params)
}
