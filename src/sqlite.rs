// database
// use crate::song::Song;
// use rusqlite::{Connection, Result};
use rusqlite::Connection;

pub struct DB {
    _conn: Connection,
}

impl Default for DB {
    fn default() -> Self {
        let conn = Connection::open("cats.db").expect("open db failed");

        conn.execute(
            "create table if not exists cat_colors (
             id integer primary key,
             name text not null unique
         )",
            [],
        )
        .expect("create table failed");
        conn.execute(
            "create table if not exists cats (
             id integer primary key,
             name text not null,
             color_id integer not null references cat_colors(id)
         )",
            [],
        )
        .expect("creat table 2 failed");
        Self { _conn: conn }
    }
}

#[allow(unused)]
impl DB {
    // pub fn db_connect(&mut self) -> Result<()> {
    //     todo!()
    // }
}
