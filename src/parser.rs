use color_eyre::eyre::Result;
use std::{fs::File, io::Read};

use crate::{error_types::TorrentParseError, model::TorrentFile};

pub fn parse_torrent_file(file_name: &str) -> Result<TorrentFile, TorrentParseError> {
    let mut file = File::open(file_name)?;

    let mut file_contents = String::new();
    file.read_to_string(&mut file_contents)?;
    let torrent_file: TorrentFile = serde_bencode::from_str(&file_contents)?;
    Ok(torrent_file)
}

#[cfg(test)]
mod test {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    pub fn test_parse_torrent_file() {
        let file_name = "archlinux-2024.09.01-x86_64.iso.torrent";
        let torrent_file = parse_torrent_file(file_name).unwrap();
        assert_ne!(None, torrent_file.info.name);
    }
}
