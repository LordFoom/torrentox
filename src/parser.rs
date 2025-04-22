use clap::crate_version;
use color_eyre::eyre::Result;
use colored::Colorize;
use log::debug;
#[allow(unused_imports)]
use log::info;
use rand::Rng;
use sha1::{Digest, Sha1};
use std::{collections::HashMap, fs::File, io::Read};

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
    let mut name = torrent_file
        .info
        .name
        .clone()
        .unwrap_or("Unknown to foom".to_string());

    name.push_str(" Top Level Container");
    let size = get_size(&torrent_file);

    let torrent = Torrent {
        torrent_file,
        name,
        file_path: file_name.to_owned(),
        raw_bytes: file_bytes,
        announce_url: None,
        downloaded: 0,
        uploaded: 0,
        size,
    };
    Ok(torrent)
}

///Get either the size (single file mode) or sume of sizes (multi filed mode)
pub fn get_size(torrent_file: &TorrentFile) -> u64 {
    if let Some(lngth) = torrent_file.info.possible_length {
        lngth
    } else if let Some(files) = torrent_file.info.possible_files.clone() {
        files.iter().map(|f| f.length).sum()
    } else {
        0
    }
}

pub fn parse_info_hash(metadata_info: &Info) -> Result<InfoHash> {
    let info_bytes = serde_bencode::to_bytes(metadata_info)?;
    let info_digest = Sha1::digest(info_bytes);
    let mut info_hash = [0; 20];
    info_hash.copy_from_slice(&info_digest);
    Ok(info_hash)
}

const UNIVERSE_OF_CHARS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890";
///If the torrentfile has a peer_id in the map, return it.
///Otherwise create a peer id and put it in the map, with the torrentfile name as the key
pub fn get_or_create_peer_id(
    torrent_file_name: &str,
    peer_id_cach: &mut HashMap<String, String>,
) -> Result<String> {
    if let Some(peer_id) = peer_id_cach.get(torrent_file_name) {
        return Ok(peer_id.to_owned());
    }
    //https://www.bittorrent.org/beps/bep_0020.html
    //first we identify ourselves - THE OX RIDES AGAIN!!
    //get our version numbers
    let version = crate_version!();
    //split it into the major/minor/tiny
    let version: Vec<&str> = version.split(".").collect();
    let major = version.first().unwrap_or(&"1");
    let minor = version.get(1).unwrap_or(&"0");
    let tiny = version.get(2).unwrap_or(&"0");
    let mut peer_id = format!("-OX{}-{}-{}-", major, minor, tiny);
    //how many more bytes do i need, count the lenght
    let remaining_chars = 20 - peer_id.len();
    let upper_bound = UNIVERSE_OF_CHARS.len();
    let mut rnd = rand::rng();

    for _ in 0..remaining_chars {
        //select a random character from our universe
        let char_num = rnd.random_range(0..upper_bound);
        let the_char = UNIVERSE_OF_CHARS.chars().nth(char_num).unwrap_or('?');
        peer_id.push(the_char);
    }
    //never again will we , the free, be subjected to the unfree freeing of the overfree
    peer_id_cach.insert(torrent_file_name.to_string(), peer_id.to_string());

    Ok(peer_id)
}

#[cfg(test)]
mod test {
    #[allow(unused_imports)]
    use super::*;

    const TORRENT_FILE_NAME: &str = "Fedora-KDE-Live-x86_64-40.torrent";
    #[allow(unused_imports)]
    use crate::log_init_for_tests;

    #[test]
    pub fn test_parse_torrent_file() {
        info!("Test of torrent file parsing is starting!");
        let file_name = TORRENT_FILE_NAME;
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
        let parse_info_hash = parse_info_hash(&md_info).unwrap();
        assert_eq!(20, parse_info_hash.len());
    }

    #[test]
    pub fn test_get_or_create_peer_id() {
        //We ask for an id whn we do not have one
        let torrent_file_name = "foom.torrent";
        let mut cache: HashMap<String, String> = HashMap::new();
        let id = get_or_create_peer_id(torrent_file_name, &mut cache).unwrap();
        info!("Id we received back: {}", id);
        //We get one back
        assert_eq!(1, cache.len());
        assert_eq!(20, id.len());
        assert!(id.starts_with("-OX"));
        //We also have it in the cache
        let default = "wrongo".to_owned();
        let cache_id = cache.get(torrent_file_name).unwrap_or(&default);
        assert_eq!(id, cache_id.to_owned());

        //we ask for an id when we have one
        cache.insert(
            "test.whatnot".to_owned(),
            "cache rules everything around me".to_owned(),
        );
        //we get the one that was in the cache back
        let new_id = get_or_create_peer_id("test.whatnot", &mut cache).unwrap();

        //there is no other one in the cache
        let second_torrent_name = "test.whatnot";
        let get_or_created_peer_id = cache.get(second_torrent_name).unwrap_or(&default);
        assert_eq!(&new_id, get_or_created_peer_id);
        //
    }

    #[test]
    pub fn test_bencoded_value() {
        let torrent = parse_torrent_file(TORRENT_FILE_NAME).unwrap();
        let bencoded = torrent.torrent_file.info_bencoded;
        info!("This is the bencoded {:?}", bencoded);
    }
}
