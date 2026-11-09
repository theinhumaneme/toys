use rusqlite::{Connection, OptionalExtension, Result};

use crate::shortner::{ShortenService, Url};

pub fn shorten(conn: &Connection, long_url: &str) -> Result<String> {
    let existing: Option<String> = conn
        .query_row(
            "SELECT short_url FROM urls WHERE long_url = ?1",
            [long_url],
            |row| row.get(0),
        )
        .optional()?;
    if let Some(short_url) = existing {
        return Ok(short_url);
    } else {
        conn.execute("INSERT INTO urls (long_url) VALUES (?1)", [long_url])?;
        let id = conn.last_insert_rowid();
        let mut url = Url {
            id,
            long_url: long_url.to_string(),
            short_url: String::new(),
        };
        url.encode();
        conn.execute(
            "UPDATE urls SET short_url = ?1 WHERE id = ?2",
            (&url.short_url, id),
        )?;
        Ok(url.short_url)
    }
}

pub fn create_tables(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS urls (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        long_url TEXT NOT NULL,
        short_url TEXT UNIQUE
        );

        CREATE UNIQUE INDEX IF NOT EXISTS idx_urls_long_url ON urls(long_url)
        ",
    )?;
    Ok(())
}

pub fn resolve(conn: &Connection, short_url: &str) -> Result<Option<String>> {
    conn.query_row(
        "SELECT long_url FROM urls WHERE short_url = ?1",
        [short_url],
        |row| row.get(0),
    )
    .optional()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        create_tables(&conn).unwrap();
        conn
    }

    #[test]
    fn shorten_then_resolve() {
        let conn = test_db();
        let code = shorten(&conn, "https://example.com").unwrap();
        assert_eq!(
            resolve(&conn, &code).unwrap(),
            Some("https://example.com".to_string())
        );
    }
}
