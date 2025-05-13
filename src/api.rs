use log::debug;
use std::collections::HashMap;

use crate::{
    database::{self, DbConnection},
    log_init_for_tests,
    model::Torrent,
    parser,
};
use color_eyre::eyre::Result;
use eyre::{eyre, Ok};

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
    debug!("Got an utf-8 info-hash? {:?}", info_hash);
    let info_hash_str = String::from_utf8_lossy(&info_hash).to_string();
    let v = urlencoding::encode(&info_hash_str);
    query_params.insert("info_hash".to_string(), v.to_string());

    let torrent_file_name = torrent_file
        .info
        .name
        .clone()
        .unwrap_or("unknown".to_string());
    query_params.insert("peer_id".to_string(), peer_id);

    let size = parser::get_size(torrent_file);
    //is this true with size always?
    let left = size - torrent.downloaded;
    //TODO get this from config
    query_params.insert("port".to_string(), "6881".to_string());
    //TODO this needs to come from DB
    query_params.insert("downloaded".to_string(), torrent.downloaded.to_string());
    query_params.insert("uploaded".to_string(), torrent.uploaded.to_string());
    query_params.insert("left".to_string(), left.to_string());

    Ok(query_params)
}

pub async fn torrent_the_files(torrent_files: &Vec<String>, db: &DbConnection) -> Result<()> {
    //log_init_for_tests::init_logging();
    let mut peer_id_cache: HashMap<String, String> = HashMap::new();
    let client = reqwest::Client::new();
    debug!("Going to loop through files: {:?}", torrent_files);
    for torrent_file_path in torrent_files {
        let torrent = parser::parse_torrent_file(&torrent_file_path)?;
        database::save_torrent_file(&torrent, db)?;
        let announce_url = torrent
            .torrent_file
            .announce
            .clone()
            .ok_or_else(|| eyre!("Did not find the announce url".to_owned()))?;
        debug!("announce url: {announce_url}");
        let query_map = construct_query_map(&torrent, &mut peer_id_cache)?;
        //create our request
        let response = client.get(announce_url).query(&query_map).send().await?;
        debug!("Our response: {:?}", response);
        let body = response.text().await?;
        debug!("Our response text: {}", body);
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    #[tokio::test]
    async fn test_torrent_the_files() {
        let torrent_files = vec!["./Fedora-KDE-Live-x86_64-40.torrent".to_string()];
        let db = database::test::init_test_conn();
        torrent_the_files(&torrent_files, &db).await.unwrap();
    }
}
