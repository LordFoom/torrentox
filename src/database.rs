use anyhow::Result;
use color_eyre::eyre::Result;
use rusqlite::Connection;

use crate::model::TorrentFile;

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

pub fn list_torrent_files(db: &DbConnection) {
    let sql = "SELECT (name, file_path, announce_url, torrent_file_raw) FROM torrent";
    db.conn.execute(sql, ())?;
}
pub fn load_torrent_file(db: &DbConnection) -> Result<Option<(TorrentFile, Vec<u8>)>> {}

#[cfg(test)]
mod test {
    use colored::*;
    use log::info;

    use super::*;

    #[test]
    fn test_init_tables() {
        let conn = Connection::open_in_memory().unwrap();
        let name = String::from("Foom test name");
        let db_name = String::from("Foom db name but it is in memory tee and, indeed, hee");
        let db = DbConnection {
            conn,
            name,
            db_name,
        };
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
}
