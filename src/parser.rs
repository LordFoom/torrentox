use color_eyre::eyre::Result;
use colored::Colorize;
use log::debug;
#[allow(unused_imports)]
use log::info;
use sha1::{Digest, Sha1};
use std::{fs::File, io::Read};

use crate::model::{Info, InfoHash, Torrent, TorrentFile};

pub fn parse_torrent_file(file_name: &str) -> Result<Torrent> {
    debug!("Parsing {file_name}");
    let mut file = File::open(file_name)?;
    debug!("Parsed {file_name}");

    debug!("Reading {file_name} into byte vec...");
    let mut file_bytes = Vec::new();
    file.read_to_end(&mut file_bytes)?;
    debug!("Read {file_name} into byte vec");
    debug!("Deserializing torrent file");
    let torrent_file: TorrentFile = serde_bencode::from_bytes(&file_bytes)?;
    debug!(
        "TorrentFile parsed - the {} rides {}",
        "Ox".truecolor(255, 165, 0).bold(),
        "again!!!".magenta().bold().italic(),
    );
    let torrent = Torrent {
        torrent_file,
        name: "Test torrent".to_owned(),
        file_path: file_name.to_owned(),
        raw_bytes: file_bytes,
        announce_url: None,
    };
    Ok(torrent)
}

pub fn parse_info_hash(metadata_info: &Info) -> Result<InfoHash> {
    let info_bytes = serde_bencode::to_bytes(metadata_info)?;
    let info_digest = Sha1::digest(info_bytes);
    let mut info_hash = [0; 20];
    info_hash.copy_from_slice(&info_digest);
    Ok(info_hash)
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
        let file_name = "Fedora-KDE-Live-x86_64-40.torrent";
        let torrent = parse_torrent_file(file_name).unwrap();
        assert_ne!(None, torrent.torrent_file.info.name);
        info!(
            "Name of this torrent info: {}",
            torrent
                .torrent_file
                .info
                .name
                .as_ref()
                .unwrap()
                .cyan()
                .bold()
        );
        info!(
            "This is the torrent file piece length: {}",
            torrent.torrent_file.info.piece_length
        );
        assert!(torrent.torrent_file.info.piece_length > 0);
        assert_eq!(None, torrent.torrent_file.info.meta_version);

        let mut files_found = false;
        let mut length_found = false;
        if let Some(files) = torrent.torrent_file.info.clone().possible_files {
            info!("Had a files element");
            info!(
                "We had {} files in the files element",
                files.len().to_string().magenta().bold()
            );
            files.iter().for_each(|file| {
                info!("File length:  {}", file.length);
                if let Some(path_vec) = file.possible_path.clone() {
                    info!("Found a path of vec of so many: {}", path_vec.len());
                    path_vec
                        .iter()
                        .for_each(|p| info!("path part: {}", p.cyan()));
                } else {
                    info!("{}", "No path was found".underline());
                }
            });
            files_found = true;
        } else {
            info!("Had NO files element");
        }

        if let Some(length) = torrent.torrent_file.info.possible_length {
            info!("We had a length element so big: {}", length);
            length_found = true;
        } else {
            info!("We had NO length element");
        }

        let exactly_one_of_two_options =
            (files_found && !length_found) || (!files_found && length_found);
        assert!(
            exactly_one_of_two_options,
            "Expected one of 'length' or 'files', but not both and not neither"
        );

        info!(
            "This is the announce url {}",
            torrent
                .torrent_file
                .announce
                .clone()
                .unwrap_or_else(|| "No announce url".to_owned())
        );

        let md_info = torrent.clone().torrent_file.info;
        parse_info_hash(&md_info).unwrap();
    }
}
