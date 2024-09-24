use std::{fs::File, io::Read};

use crate::{error_types::TorrentParseError, model::TorrentFile};

pub fn parse_torrent_file(file_name: &str) -> Result<TorrentFile, TorrentParseError> {
    let mut file = File::from(file_name);

    let mut file_contents = String::new();
    file.read_to_string(&mut file_contents);
    let torrent_file: TorrentFile = serde_bencode::from_str(&file_contents)?;
    Ok(torrent_file)
}

#[cfg(test)]
mod test {}
