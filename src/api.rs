use std::{collections::HashMap, str};

use crate::{model::Torrent, parser};
use color_eyre::eyre::Result;
use eyre::Ok;

///The call to the announce url is an HTTP request
pub fn construct_query_map(
    torrent: &Torrent,
    peer_id_cache: &mut HashMap<String, String>,
) -> Result<HashMap<String, String>> {
    //here we get the peer_id
    let torrent_file = &torrent.torrent_file;
    let name = torrent_file
        .info
        .name
        .clone()
        .unwrap_or("wrongo".to_owned());
    let peer_id = parser::get_or_create_peer_id(&name, peer_id_cache)?;
    //we construct a map of param = > value
    let mut query_params = HashMap::new();
    let info_hash = parser::parse_info_hash(&torrent_file.info)?;
    let info_hash_str = str::from_utf8(&info_hash)?;
    let v = urlencoding::encode(info_hash_str);
    query_params.insert("info_hash".to_string(), v.to_string());

    let torrent_file_name = torrent_file
        .info
        .name
        .clone()
        .unwrap_or("unknown".to_string());
    query_params.insert("peer_id".to_string(), peer_id);

    //TODO get this from config
    query_params.insert("port".to_string(), "6881".to_string());
    //TODO this needs to come from DB
    query_params.insert("downloaded".to_string(), "0".to_string());
    query_params.insert("left".to_string(), torrent.left.to_string());

    Ok(query_params)
}
