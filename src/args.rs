///CLI arguments we can pass the application
#[derive(Debug, Parser)]
struct AppArgs {
    ///List of arguments we can give
    //the reviled java programmer
    #[arg(s)]
    torrent_files: Vec<String>,
}
