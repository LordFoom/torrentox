mod args;
mod database;
mod error_types;
mod log_init_for_tests;
mod model;
mod parser;
use clap::Parser;

use args::AppArgs;
use color_eyre::eyre::Result;
use database::{init_tables, save_torrent_file, DbConnection};
use log::LevelFilter;
use log4rs::{
    append::file::FileAppender,
    config::{runtime::Appender, Logger, Root},
    encode::pattern::PatternEncoder,
    Config,
};
use parser::parse_torrent_file;
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
fn main() -> Result<()> {
    let args = AppArgs::parse();
    init(args.verbose)?;

    let db = init_db()?;
    init_tables(&db)?;

    let torrent_files = args.torrent_files;
    for torrent_file_path in torrent_files {
        let (torrent, bytes) = parse_torrent_file(&torrent_file_path)?;
        save_torrent_file(&torrent, &torrent_file_path, &bytes, &db)?;
    }

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
