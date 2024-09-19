///CLI arguments we can pass the application
#[derive(Debug, Parser)]
pub struct AppArgs {
    ///List of arguments we can give
    //the reviled java programmer
    pub torrent_files: Vec<String>,
}
