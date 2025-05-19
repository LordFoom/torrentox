use log::debug;
use serde_bencode::de;
use std::collections::HashMap;
use url::form_urlencoded;

use crate::{
    database::{self, DbConnection},
    model::{InfoHash, Torrent, TorrentFile, TrackerAnnounceResponse},
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

    //let torrent_file_name = torrent_file
    //    .info
    //    .name
    //    .clone()
    //    .unwrap_or("unknown".to_string());
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
    //TODO make this a tui and such
    //once we get the loading of the down working
    for torrent_file_path in torrent_files {
        let torrent = parser::parse_torrent_file(&torrent_file_path)?;
        database::save_torrent_file(&torrent, db)?;
        let announce_url = torrent
            .torrent_file
            .announce
            .clone()
            .ok_or_else(|| eyre!("Did not find the announce url".to_owned()))?;
        debug!("announce url: {announce_url}");

        let info_hash = &torrent.torrent_file.info_hash;
        let encoded_info_hash: String = form_urlencoded::byte_serialize(info_hash).collect();

        let query_map = construct_query_map(&torrent, &mut peer_id_cache)?;
        let encoded_params = serde_urlencoded::to_string(query_map)?;
        //create our request
        let full_announce_url = format!(
            "{}?{}&info_hash={}",
            announce_url,
            encoded_params.clone(),
            encoded_info_hash,
        );
        debug!("full_announce_url={full_announce_url}");
        let response = client
            .get(full_announce_url)
            .send()
            .await?
            .error_for_status()?;
        debug!("Our response: {:?}", response);

        let http_status = response.status();
        if http_status.is_server_error() {
            let body = response.text().await?;
            let err = eyre!(
                "Server error, {}, with message {}",
                http_status.to_string(),
                body
            );
            return Err(err);
        } else if http_status.is_client_error() {
            let body = response.text().await?;
            let err = eyre!(
                "Client error, {}, with message {}",
                http_status.to_string(),
                body
            );
            return Err(err);
        }
        //debug!("Our response text: {}", body);

        let body_bytes = response.bytes().await?;
        let response: TrackerAnnounceResponse = de::from_bytes(&body_bytes)?;
        //we get to here it was successful
        //extract the peers from the response text
        //let peers = extract_peers
    }
    Ok(())
}

//fn construct_info_hash(torrent_file: &TorrentFile) -> Result<InfoHash> {
//    let info_hash = parser::parse_info_hash(&torrent_file.info)?;
//    debug!("Got an utf-8 info-hash? {:?}", info_hash);
//    //let info_hash_str = String::from_utf8_lossy(&info_hash).to_string();
//    //let v = urlencoding::encode(&info_hash_str);
//    //query_params.insert("info_hash".to_string(), v.to_string());
//    Ok(info_hash)
//}

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
