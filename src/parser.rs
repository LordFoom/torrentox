use color_eyre::eyre::Result;
use log::debug;
use log::info;
use std::{fs::File, io::Read};

use crate::{error_types::TorrentParseError, model::TorrentFile};

pub fn parse_torrent_file(file_name: &str) -> Result<TorrentFile, TorrentParseError> {
    debug!("Parsing {file_name}");
    let mut file = File::open(file_name)?;
    debug!("Parsed {file_name}");

    debug!("Reading {file_name} into byte vec...");
    let mut file_bytes = Vec::new();
    file.read_to_end(&mut file_bytes)?;
    debug!("Read {file_name} into byte vec");
    debug!("Deserializing torrent file");
    let torrent_file: TorrentFile = serde_bencode::from_bytes(&file_bytes)?;
    debug!("TorrentFile parsed - the Ox rides again!");
    Ok(torrent_file)
}

#[cfg(test)]
mod test {
    #[allow(unused_imports)]
    use super::*;

    #[allow(unused_imports)]
    use crate::log_init_for_tests;

    #[test]
    pub fn test_parse_torrent_file() {
        info!("Test is starting!");
        let file_name = "archlinux-2024.09.01-x86_64.iso.torrent";
        let torrent_file = parse_torrent_file(file_name).unwrap();
        assert_ne!(None, torrent_file.info.name);
    }
}
