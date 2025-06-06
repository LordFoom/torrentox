use log::debug;
use serde_bencode::de;
use std::collections::HashMap;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use url::form_urlencoded;

use crate::model::TorrentSession;
use crate::{
    database::{self, DbConnection},
    model::{Handshake, InfoHash, Peer, PeerId, Torrent, TorrentFile, TrackerAnnounceResponse},
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

pub async fn init_peer_torrent_sessions(
    torrent_files: &Vec<String>,
    db: &DbConnection,
) -> Result<Vec<TorrentSession>> {
    //log_init_for_tests::init_logging();
    let mut peer_id_cache: HashMap<String, String> = HashMap::new();
    let client = reqwest::Client::new();
    debug!("Going to loop through files: {:?}", torrent_files);
    //TODO make this a tui and such
    //once we get the loading of the down working
    let mut torrents: Vec<TorrentSession> = Vec::new();
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
        let peer_id_str = query_map.get("peer_id").ok_or_else(|| {
            eyre!("Expected peer_id to be assigned in the query_map by now".to_string())
        })?;
        let peer_id_bytes = peer_id_str.as_bytes();
        if peer_id_bytes.len() != 20 {
            return Err(eyre!("Peer Id must be exactly 20 bytes long"));
        }
        let mut peer_id = [0u8; 20];
        peer_id.copy_from_slice(peer_id_bytes);
        let encoded_params = serde_urlencoded::to_string(&query_map)?;
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
        //if we get to here it was successful
        response
            .peers
            .iter()
            .for_each(|peer| debug!("Peer! {}", peer));
        let ts = TorrentSession {
            peer_id,
            peers: response.peers,
            torrent,
        };
        torrents.push(ts);
    }
    Ok(torrents)
}

pub async fn connect_and_send_handshake(
    peer_ip: &str,
    peer_port: u16,
    info_hash: &[u8; 20],
    peer_id: &[u8; 20],
) -> Result<()> {
    log.debug("connect_and_send_handshake firing...");
    let addr = format!("{}:{}", peer_ip, peer_port);
    let mut stream = TcpStream::connect(addr).await?;

    // Build the handshake
    let handshake = build_handshake(info_hash, peer_id);

    // Send the handshake
    stream.write_all(&handshake).await?;

    // Read the peer's handshake response (68 bytes)
    let mut response = [0u8; 68];
    stream.read_exact(&mut response).await?;

    // You can now parse the response using Handshake::from_bytes
    println!("Received handshake: {:?}", &response[..]);

    log.debug("connect_and_send_handshake finished.");
    Ok(())
}

pub fn build_handshake(info_hash: &InfoHash, peer_id: &PeerId) -> Handshake {
    let mut handshake = [0u8; 68];
    //Protocol string length (always 19)
    handshake[0] = 19;
    //Protocol string "BitTorrent protocol"
    handshake[1..20].copy_from_slice(b"BitTorrent protocol");
    // Reserved bytes (8 bytes, usually all zeros unless supporting extensions)
    //   // Already zeroed by array initialization
    //Info hash (20 bytes)
    handshake[28..48].copy_from_slice(info_hash);
    handshake[48..68].copy_from_slice(peer_id);
    handshake
}

#[cfg(test)]
mod test {
    use super::*;
    #[tokio::test]
    async fn test_get_peer_list() {
        let torrent_files = vec!["./Fedora-KDE-Live-x86_64-40.torrent".to_string()];
        let db = database::test::init_test_conn();
        init_peer_torrent_sessions(&torrent_files, &db)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_connect_and_send_handshake() {
        let torrent_files = vec!["./Fedora-KDE-Live-x86_64-40.torrent".to_string()];
        let db = database::test::init_test_conn();
        let torrent_sessions = init_peer_torrent_sessions(&torrent_files, &db)
            .await
            .unwrap();
        for torrent_session in torrent_sessions {
            for peer in torrent_session.peers {
                connect_and_send_handshake(
                    &peer.ip,
                    peer.port,
                    &torrent_session.torrent.torrent_file.info_hash,
                    &torrent_session.peer_id,
                )
                .await
                .unwrap()
            }
        }
    }
}
