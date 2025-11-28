use spin_sdk::sqlite::{Connection, Value};
use anyhow::{Result, Context};
use crate::models::ImageMetadata;

pub fn init(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS images (
            id TEXT PRIMARY KEY,
            image BLOB NOT NULL,
            mime_type TEXT NOT NULL,
            hash TEXT NOT NULL,
            name TEXT,
            description TEXT
        )",
        &[],
    )?;
    
    // Migration for existing tables
    let _ = conn.execute("ALTER TABLE images ADD COLUMN name TEXT", &[]);
    let _ = conn.execute("ALTER TABLE images ADD COLUMN description TEXT", &[]);
    
    Ok(())
}

pub fn insert_image(
    conn: &Connection, 
    id: &str, 
    image_data: &[u8], 
    mime_type: &str, 
    hash: &str, 
    name: &str, 
    description: &str
) -> Result<()> {
    conn.execute(
        "INSERT INTO images (id, image, mime_type, hash, name, description) VALUES (?, ?, ?, ?, ?, ?)",
        &[
            Value::Text(id.to_string()),
            Value::Blob(image_data.to_vec()),
            Value::Text(mime_type.to_string()),
            Value::Text(hash.to_string()),
            Value::Text(name.to_string()),
            Value::Text(description.to_string()),
        ],
    )?;
    Ok(())
}

pub fn get_image_data(conn: &Connection, id: &str) -> Result<Option<(Vec<u8>, String, String)>> {
    let result = conn.execute(
        "SELECT image, mime_type, hash FROM images WHERE id = ?",
        &[Value::Text(id.to_string())],
    )?;
    
    let mut rows = result.rows();
    if let Some(row) = rows.next() {
        let image = row.get::<&[u8]>("image").context("getting image")?.to_vec();
        let mime_type = row.get::<&str>("mime_type").context("getting mime_type")?.to_string();
        let hash = row.get::<&str>("hash").context("getting hash")?.to_string();
        Ok(Some((image, mime_type, hash)))
    } else {
        Ok(None)
    }
}

pub fn get_random_image(conn: &Connection) -> Result<Option<ImageMetadata>> {
    let result = conn.execute(
        "SELECT id, name, description FROM images ORDER BY RANDOM() LIMIT 1",
        &[],
    )?;
    
    row_to_metadata(result)
}

pub fn get_image_metadata(conn: &Connection, id: &str) -> Result<Option<ImageMetadata>> {
    let result = conn.execute(
        "SELECT id, name, description FROM images WHERE id = ?",
        &[Value::Text(id.to_string())],
    )?;
    
    row_to_metadata(result)
}

pub fn delete_image(conn: &Connection, id: &str) -> Result<bool> {
    // Check if the image exists first
    let result = conn.execute(
        "SELECT id FROM images WHERE id = ?",
        &[Value::Text(id.to_string())],
    )?;

    let mut rows = result.rows();
    if rows.next().is_none() {
        // Nothing to delete
        return Ok(false);
    }

    // Perform the delete
    let _ = conn.execute(
        "DELETE FROM images WHERE id = ?",
        &[Value::Text(id.to_string())],
    )?;

    Ok(true)
}

fn row_to_metadata(result: spin_sdk::sqlite::QueryResult) -> Result<Option<ImageMetadata>> {
    let mut rows = result.rows();
    if let Some(row) = rows.next() {
        let id = row.get::<&str>("id").context("getting id")?.to_string();
        let name = row.get::<&str>("name").unwrap_or("Unknown").to_string();
        let description = row.get::<&str>("description").unwrap_or("No description").to_string();
        let url = format!("/api/image/{}", id);
        
        Ok(Some(ImageMetadata {
            id,
            url,
            name,
            description,
        }))
    } else {
        Ok(None)
    }
}
