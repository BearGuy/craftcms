use crate::models::{Image, User};
use rusqlite::{params, Connection, Error};
use uuid::Uuid;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

pub fn init_db() -> Result<Connection, Error> {
    let conn = Connection::open("data/craftcms.db")?;
    Ok(conn)
}

pub fn run_migrations(conn: &Connection) -> Result<(), Error> {
    conn.execute_batch(
        r#"

        CREATE TABLE IF NOT EXISTS images (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            alt TEXT NOT NULL,
            description TEXT,
            slug TEXT UNIQUE NOT NULL,
            keywords TEXT,
            file_path TEXT NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            email TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id TEXT UNIQUE NOT NULL,
            user_id INTEGER NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            expires_at DATETIME NOT NULL,
            FOREIGN KEY(user_id) REFERENCES users(id)
        );
        "#,
    )?;
    Ok(())
}

// User operations
pub fn create_user(
    conn: &Connection,
    email: &str,
    password: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Generate a random salt
    let salt = SaltString::generate(&mut OsRng);

    // Create Argon2 instance
    let argon2 = Argon2::default();

    // Hash the password
    let password_hash = match argon2.hash_password(password.as_bytes(), &salt) {
        Ok(hash) => hash.to_string(),
        Err(e) => {
            return Err(Box::new(rusqlite::Error::InvalidParameterName(
                e.to_string(),
            )))
        }
    };

    // Store the user with the hashed password
    conn.execute(
        "INSERT INTO users (email, password_hash) VALUES (?, ?)",
        [email, &password_hash],
    )?;

    Ok(())
}

pub fn list_users(conn: &Connection) -> rusqlite::Result<Vec<User>> {
    let mut stmt = conn.prepare("SELECT id, email FROM users")?;
    let users = stmt.query_map([], |row| {
        Ok(User {
            id: row.get(0)?,
            email: row.get(1)?,
            password_hash: String::new(), // We don't need to return this
            access_token: None,
        })
    })?;

    users.collect()
}

pub fn delete_user(conn: &Connection, email: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM users WHERE email = ?", [email])?;
    Ok(())
}

pub fn verify_user(
    conn: &Connection,
    email: &str,
    password: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let hash: String = conn.query_row(
        "SELECT password_hash FROM users WHERE email = ?",
        [email],
        |row| row.get(0),
    )?;

    // Parse the stored hash string
    let parsed_hash = match PasswordHash::new(&hash) {
        Ok(hash) => hash,
        Err(e) => {
            return Err(Box::new(rusqlite::Error::InvalidParameterName(
                e.to_string(),
            )))
        }
    };

    // Verify the password
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

// Session operations
pub fn create_session(conn: &Connection, user_id: i64) -> Result<String, Error> {
    let session_id = Uuid::new_v4().to_string();
    let expires_at = (chrono::Utc::now() + chrono::Duration::hours(24))
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();

    conn.execute(
        "INSERT INTO sessions (session_id, user_id, expires_at) VALUES (?, ?, ?)",
        params![session_id, user_id, expires_at],
    )?;

    Ok(session_id)
}

pub fn verify_session(conn: &Connection, session_id: &str) -> Result<bool, Error> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sessions WHERE session_id = ? AND expires_at > datetime('now')",
        [session_id],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

pub fn delete_session(conn: &Connection, session_id: &str) -> Result<(), Error> {
    conn.execute("DELETE FROM sessions WHERE session_id = ?", [session_id])?;
    Ok(())
}

pub fn get_images(conn: &Connection) -> Result<Vec<Image>, Error> {
    let mut stmt = conn.prepare(
        "SELECT alt, description, slug, keywords, filename FROM images ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map(params![], |row| {
        Ok(Image {
            alt: row.get(0)?,
            description: row.get(1)?,
            slug: row.get(2)?,
            keywords: serde_json::from_str(&row.get::<_, String>(3)?).unwrap_or_default(),
            filename: row.get(4)?,
        })
    })?;

    rows.collect()
}

pub fn get_image_by_slug(conn: &Connection, slug: &str) -> Result<Image, Error> {
    let mut stmt = conn
        .prepare("SELECT alt, description, slug, keywords, filename FROM images WHERE slug = ?")?;

    stmt.query_row(params![slug], |row| {
        Ok(Image {
            alt: row.get(0)?,
            description: row.get(1)?,
            slug: row.get(2)?,
            keywords: serde_json::from_str(&row.get::<_, String>(3)?).unwrap_or_default(),
            filename: row.get(4)?,
        })
    })
}

pub fn insert_image(conn: &Connection, image: &Image) -> Result<(), Error> {
    conn.execute(
        "INSERT INTO images (alt, description, slug, keywords, filename)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            &image.alt,
            &image.description,
            &image.slug,
            serde_json::to_string(&image.keywords).unwrap(),
            &image.filename,
        ],
    )?;
    Ok(())
}

pub fn update_image(conn: &Connection, slug: &str, image: &Image) -> Result<(), Error> {
    conn.execute(
        "UPDATE images SET alt = ?1, description = ?2, slug = ?3, keywords = ?4, image_data = ?5 WHERE slug = ?6",
        params![
            &image.alt,
            &image.description,
            &image.slug,
            serde_json::to_string(&image.keywords).unwrap(),
            &image.filename,
            slug,
        ],
    )?;
    Ok(())
}

pub fn delete_image(conn: &Connection, slug: &str) -> Result<(), Error> {
    conn.execute("DELETE FROM images WHERE slug = ?", params![slug])?;
    Ok(())
}
