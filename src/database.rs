use color_eyre::eyre::Result;
use rusqlite::Connection;

///Holder of the DB Connection information
struct DbConnection {
    pub conn: Connection,
    ///Name of the db on the file
    pub db_name: String,
    pub name: String,
}
///Create necessary torrent tables iff not already created.
pub fn init_tables(db: &DbConnection) -> Result<()> {
    let create_table = "CREATE TABLE IF NOT EXISTS torrent (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT, file_path TEXT, announce_url TEXT, torent_file_contents BLOB) ";
    db.conn.execute(create_table, [])?;
    Ok(())
}
