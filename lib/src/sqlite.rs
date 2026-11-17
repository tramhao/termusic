use rusqlite::Connection;

/// Configure settings required by library and podcast database connections.
pub(crate) fn configure_connection(conn: &Connection) -> rusqlite::Result<()> {
    // We force bundling and rusqlite enables this by default, but systems like nixos can force system library usage, which may or may not have this enabled by default.
    conn.pragma_update(None, "foreign_keys", true)
}
