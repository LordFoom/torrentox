mod api;
mod args;
mod database;
mod error_types;
mod log_init_for_tests;
mod model;
mod parser;

use api::get_peer_list;
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
    let peer_torrent = get_peer_list(&torrent_files, &db).await?;

    let client = reqwest::Client::new();
    for (peer_id, peer_list) in peer_torrent {
        //get a response from the peer
        for peer in peer_list {
            let peer_url = format!("http://{}:{}", peer.ip, peer.port);
            let response = client.get(peer_url).send().await?.error_for_status();
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
