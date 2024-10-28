use anyhow::anyhow;
use color_eyre::eyre::{Result, WrapErr};
use rusqlite::params;
use rusqlite::Connection;
use rusqlite::Error as RusqliteError;

use crate::error_types::AppError;
use crate::error_types::DbError;
use crate::error_types::TorrentParseError;
use crate::model::{Torrent, TorrentFile};

///Holder of the DB Connection information
pub struct DbConnection {
    pub conn: Connection,
    ///Name of the db on the file
    pub db_name: String,
    pub name: String,
}
///Create necessary torrent tables iff not already created.
pub fn init_tables(db: &DbConnection) -> Result<()> {
    let create_table = "CREATE TABLE IF NOT EXISTS torrent (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT, file_path TEXT, announce_url TEXT, torrent_file_raw BLOB) ";
    db.conn.execute(create_table, [])?;
    Ok(())
}

///We save a torrent file, recording a few attributes,
///but otherwise storing the raw bytes so as not to lose any info during the coding process
pub fn save_torrent_file(
    torrent_file: &TorrentFile,
    file_path: &str,
    raw_bytes: &Vec<u8>,
    db: &DbConnection,
) -> Result<()> {
    let sql = "INSERT INTO torrent (name, file_path, announce_url, torrent_file_raw) VALUES (?1, ?2, ?3, ?4) ";
    db.conn.execute(
        sql,
        (
            torrent_file.info.name.clone().unwrap_or("None".to_owned()),
            file_path,
            torrent_file.announce.clone().unwrap_or("None".to_owned()),
            raw_bytes,
        ),
    )?;
    Ok(())
}

pub fn list_torrent_files(db: &DbConnection) -> Result<Vec<Torrent>> {
    let sql = "SELECT name, file_path, announce_url, torrent_file_raw FROM torrent";
    let mut stmt = db
        .conn
        .prepare(sql)
        .map_err(DbError::from)
        .wrap_err("Failed to prepare the list torrent file statement")?;
    //let torrent_file_list = stmt.query_map([], |row| {
    //    let torrent_file_raw: Vec<u8> = row.get(3)?;
    //    let torrent_file = serde_bencode::from_bytes(&torrent_file_raw)
    //        .map_err("Failed to create torrent file from db bytes")?;
    //    //let torrent_file =
    //    //    serde_bencode::from_bytes(&torrent_file_raw).map_err(|err| rusqlite::Error(err));
    //    //(row.get(0), row.get(1), row.get(2), row.get(3)));
    //});
    //let vec = Vec::new();
    //Ok(vec)
    let torrent_file_list = stmt
        .query_map([], |row| {
            let torrent_file_raw: Vec<u8> = row.get(3)?;
            let torrent_file = serde_bencode::from_bytes(&torrent_file_raw).unwrap();
            //.map_err(DbError::from)
            //.wrap_err("Failed to deserialize torrent_file_raw")?;

            Ok(Torrent {
                name: row.get(0)?,
                file_path: row.get(1)?,
                announce_url: row.get(2)?,
                torrent_file,
            })
        })
        .wrap_err("Failed to map query result")?;
    let mut torrent_vec = Vec::new();

    for torrent in torrent_file_list {
        let tr = torrent.wrap_err("Could not retrieve the torrent bytes from db")?;
        torrent_vec.push(tr);
    }

    Ok(torrent_vec)
}

///Use the name. Get the file
pub fn load_torrent_file(name: &str, db: &DbConnection) -> Result<Torrent> {
    let sql = "SELECT name, file_path, announce_url, torrent_file_raw FROM torrent where name = ?1";
    db.conn
        .query_row(sql, params![name], |row| {
            let torrent_file_raw: Vec<u8> = row.get(3)?;
            let torrent_file = serde_bencode::from_bytes(&torrent_file_raw).unwrap();

            Ok(Torrent {
                name: row.get(0)?,
                file_path: row.get(1)?,
                announce_url: row.get(2)?,
                torrent_file,
            })
        })
        .wrap_err("")
}

#[cfg(test)]
mod test {
    use colored::*;
    use log::info;

    use super::*;

    #[test]
    fn test_init_tables() {
        let db = init_test_conn();
        init_tables(&db).unwrap();
        //does our table exist
        let mut stmt = db
            .conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table'")
            .unwrap();
        let mut tables: Vec<String> = Vec::new();
        let table_rows = stmt.query_map([], |row| row.get(0)).unwrap();
        for table_name in table_rows {
            tables.push(table_name.unwrap());
        }
        info!(
            "num tables found: {}",
            tables.len().to_string().magenta().bold()
        );

        assert!(tables.contains(&String::from("torrent")));
    }

    fn init_test_conn() -> DbConnection {
        let conn = Connection::open_in_memory().unwrap();
        let name = String::from("Foom test name");
        let db_name = String::from("Foom db name but it is in memory tee and, indeed, hee");
        let db = DbConnection {
            conn,
            name,
            db_name,
        };
        db
    }

    #[test]
    fn test_save_torrent_file() {
        let conn = init_test_conn();
    }
}
