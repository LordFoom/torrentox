mod api;
mod args;
mod database;
mod error_types;
mod log_init_for_tests;
mod model;
mod parser;

use api::connect_and_send_handshake;
use api::init_peer_torrent_sessions;
use clap::Parser;

//use anyhow::Result;
use args::AppArgs;
use color_eyre::eyre::{eyre, Result};
use database::{init_tables, save_torrent_file, DbConnection};
use log::debug;
use log::LevelFilter;
use log4rs::{
    append::file::FileAppender,
    config::{runtime::Appender, Logger, Root},
    encode::pattern::PatternEncoder,
    Config,
};
use rusqlite::Connection;

fn init(verbose: bool) -> Result<()> {
    //pretty error messages
    color_eyre::install()?;

    let log_level = if verbose {
        LevelFilter::Debug
    } else {
        LevelFilter::Warn
    };

    //logging

    let file_log = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d} {l} - [{f}]>>{M}:{L} {m}{n}",
        )))
        .build("torrentox.log")?;

    let file_appender_name = "file_log";
    let config = Config::builder()
        .appender(Appender::builder().build(file_appender_name, Box::new(file_log)))
        .logger(
            Logger::builder()
                .appender(file_appender_name)
                .additive(false)
                .build("app::file_log", log_level),
        )
        .build(
            Root::builder()
                .appender(file_appender_name)
                .build(log_level),
        )?;

    //this bad boy can enable us to change logging config at runtime, which i can think would
    //be nice, to be able to go from one to the other, hmmmmmmmmm
    let _handle = log4rs::init_config(config)?;
    debug!("Logging initialized. 🦋🪵🪓🪚");

    Ok(())
}
///TODO add torrent file parsing
///TODO add peer retrieval
///TODO add peer connection
///TODO add downloading from peer
///TODO seeding
///TODO resume downloading partially downloaded file
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args = AppArgs::parse();
    init(args.verbose)?;

    let db = init_db()?;
    init_tables(&db)?;

    let torrent_files = args.torrent_files;
    let peer_torrent = init_peer_torrent_sessions(&torrent_files, &db).await?;

    let client = reqwest::Client::new();
    for (torrent_session) in peer_torrent {
        //get a response from the peer
        for peer in torrent_session.peers {
            let peer_url = format!("http://{}:{}", peer.ip, peer.port);
            connect_and_send_handshake(
                &peer.ip,
                peer.port,
                &torrent_session.torrent.torrent_file.info_hash,
                &torrent_session.peer_id,
            )
            .await?;
            // let response = client.get(peer_url).send().await?;
            // let http_status = response.status();
            // //handle error status
            // if http_status.is_server_error() {
            //     let body = response.text().await?;
            //     let err = eyre!(
            //         "Server error, {}, with message {}",
            //         http_status.to_string(),
            //         body
            //     );
            //     return Err(err);
            // } else if http_status.is_client_error() {
            //     let body = response.text().await?;
            //     let err = eyre!(
            //         "Client error, {}, with message {}",
            //         http_status.to_string(),
            //         body
            //     );
            //     return Err(err);
            // }

            // let txt = response.text().await.error
        }
    }
    //TODO these should come from the db and be stored there
    //let mut peer_id_cache: HashMap<String, String> = HashMap::new();
    //
    //let client = reqwest::Client::new();
    //for torrent_file_path in torrent_files {
    //    let torrent = parse_torrent_file(&torrent_file_path)?;
    //    save_torrent_file(&torrent, &db)?;
    //    let announce_url = torrent
    //        .torrent_file
    //        .announce
    //        .clone()
    //        .ok_or_else(|| eyre!("Did not find the announce url"))?;
    //    debug!("announce url: {announce_url}");
    //    let query_map = construct_query_map(&torrent, &mut peer_id_cache)?;
    //    //create our request
    //    let response = client.get(announce_url).query(&query_map).send().await?;
    //    debug!("Our response: {:?}", response);
    //}
    //connect to the announce url

    Ok(())
}

fn init_db() -> Result<DbConnection> {
    let conn = Connection::open("./torrentox.db")?;
    let db = DbConnection {
        conn,
        name: "TorrentOx".to_owned(),
        db_name: "torrentox.db".to_owned(),
    };
    Ok(db)
}
