use clap::Parser;
///CLI arguments we can pass the application
#[derive(Debug, Parser)]
#[command(version, about, long_about = "CLI rust based torrent TUI")]
pub struct AppArgs {
    ///List of arguments we can give
    //the reviled java programmer
    pub torrent_files: Vec<String>,
}
