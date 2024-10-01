mod args;
mod error_types;
mod model;
mod parser;
use clap::Parser;

use args::AppArgs;
use color_eyre::eyre::Result;
use parser::parse_torrent_file;

fn init(verbose: &bool) -> Result<()> {
    //pretty error messages
    color_eyre::install()?;

    //logging
    let stdout = 
    Ok(())
}
///TODO add torrent file parsing
///TODO add peer retrieval
///TODO add peer connection
///TODO add downloading from peer
///TODO seeding
///TODO resume downloading partially downloaded file
fn main() -> Result<()> {
    init()?;
    let args = AppArgs::parse();
    let torrent_files = args.torrent_files;
    for torrent_file in torrent_files {
        let parsed_file = parse_torrent_file(&torrent_file);
    }

    Ok(())
}
