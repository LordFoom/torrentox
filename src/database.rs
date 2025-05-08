use crate::error_types::DbError;
use crate::model::Torrent;
use color_eyre::eyre::{Result, WrapErr};
use log::info;
use rusqlite::params;
use rusqlite::Connection;

///Holder of the DB Connection information
pub struct DbConnection {
    pub conn: Connection,
    ///Name of the db on the file
    pub db_name: String,
    pub name: String,
}
///Create necessary torrent tables iff not already created.
pub fn init_tables(db: &DbConnection) -> Result<()> {
    //TODO: add (bytes)downloaded (bytes)left fields
    let create_table_torrent= "CREATE TABLE IF NOT EXISTS torrent (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT, file_path TEXT, announce_url TEXT, torrent_file_raw BLOB, file_size INTEGER, downloaded INTEGER, uploaded INTEGER) ";
    let create_table_torrent_file = "CREATE TABLE IF NOT EXISTS torrent_file(id INTEGER PRIMARY KEY AUTOINCREMENT, torrent_id INTEGER, path TEXT, size INTEGER, downloaded INTEGER, uploaded INTEGER, FOREIGN KEY (torrent_id) REFERENCES torrent(id) ON DELETE CASCADE) ";
    db.conn.execute(create_table_torrent, [])?;
    db.conn.execute(create_table_torrent_file, [])?;
    Ok(())
}

///We save a torrent file, recording a few attributes,
///but otherwise storing the raw bytes so as not to lose any info during the coding process
pub fn save_torrent_file(torrent: &Torrent, db: &DbConnection) -> Result<()> {
    let sql = "INSERT INTO torrent (name, file_path, announce_url, torrent_file_raw) VALUES (?1, ?2, ?3, ?4) ";
    db.conn.execute(
        sql,
        (
            torrent
                .torrent_file
                .info
                .name
                .clone()
                .unwrap_or("None".to_owned()),
            torrent.file_path.clone(),
            torrent
                .torrent_file
                .announce
                .clone()
                .unwrap_or("None".to_owned()),
            torrent.raw_bytes.clone(),
        ),
    )?;
    Ok(())
}

pub fn list_torrent_files(db: &DbConnection) -> Result<Vec<Torrent>> {
    //are we going to have to split this up by file IN the torrent?
    //do we need a child table that has the actual files in it?
    //yes we do
    let sql = "SELECT name, file_path, announce_url, torrent_file_raw, size, downloaded, uploaded FROM torrent";
    let mut stmt = db
        .conn
        .prepare(sql)
        .map_err(DbError::from)
        .wrap_err("Failed to prepare the list torrent file statement")?;
    let torrent_file_list = stmt
        .query_map([], |row| {
            let torrent_file_raw: Vec<u8> = row.get(3)?;
            let torrent_file = serde_bencode::from_bytes(&torrent_file_raw).unwrap();
            Ok(Torrent {
                name: row.get(0)?,
                file_path: row.get(1)?,
                announce_url: row.get(2)?,
                torrent_file,
                raw_bytes: torrent_file_raw,
                size: row.get(4)?,
                downloaded: row.get(5)?,
                uploaded: row.get(6)?,
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
pub fn select_torrent_file(name: &str, db: &DbConnection) -> Result<Torrent> {
    let sql = "SELECT name, file_path, announce_url, torrent_file_raw, downloaded, uploaded, size FROM torrent where name = ?1";
    info!("Our select statment: {sql}");
    info!("The name we will use: {name}");
    db.conn
        .query_row(sql, params![name], |row| {
            let torrent_file_raw: Vec<u8> = row.get(3)?;
            let torrent_file = serde_bencode::from_bytes(&torrent_file_raw).unwrap();

            Ok(Torrent {
                name: row.get(0)?,
                file_path: row.get(1)?,
                announce_url: row.get(2)?,
                torrent_file,
                raw_bytes: torrent_file_raw,
                downloaded: row.get(4)?,
                uploaded: row.get(5)?,
                size: row.get(5)?,
            })
        })
        .wrap_err("Error retrieving the torrent by name")
}

#[cfg(test)]
pub mod test {
    use colored::*;
    use log::info;

    use crate::parser::parse_torrent_file;

    use super::*;

    #[test]
    fn test_init_tables() {
        let db = init_test_conn();
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

    pub fn init_test_conn() -> DbConnection {
        let conn = Connection::open_in_memory().unwrap();
        let name = String::from("Foom test name");
        let db_name = String::from("Foom db name but it is in memory tee and, indeed, hee");
        let db = DbConnection {
            conn,
            name,
            db_name,
        };
        init_tables(&db).unwrap();
        db
    }

    #[test]
    fn test_save_torrent_file() {
        let db = init_test_conn();
        let file_name = "Fedora-KDE-Live-x86_64-40.torrent";
        let torrent = parse_torrent_file(file_name).unwrap();

        save_torrent_file(&torrent, &db).unwrap();

        let torrents = list_torrent_files(&db).unwrap();
        assert_eq!(1, torrents.len());

        torrents
            .iter()
            .for_each(|torrent| info!("This is the name of the torrent: {}", torrent.name));

        let torrent_name_to_test = file_name.replace(".torrent", "");
        let retrieved_torrent = select_torrent_file(&torrent_name_to_test, &db).unwrap();
        assert_eq!(torrent.announce_url, retrieved_torrent.announce_url);
    }
}
