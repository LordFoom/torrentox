mod args;
mod error_types;
mod model;
mod parser;

use args::AppArgs;
///TODO add torrent file parsing
///TODO add peer retrieval
///TODO add peer connection
///TODO add downloading from peer
///TODO seeding
///TODO resume downloading partially downloaded file
fn main() {
    let args = AppArgs::parse();
    let torrent_files = args.torrent_files;
}
