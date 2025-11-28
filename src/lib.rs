mod db;
mod models;
mod auth;

use spin_sdk::http::{IntoResponse, Request, Response, Method};
use spin_sdk::http_component;
use spin_sdk::sqlite::Connection;
use uuid::Uuid;
use sha2::{Sha256, Digest};
use bytes::Bytes;
use multipart::server::Multipart;
use std::io::Read;
use models::UploadResponse;

#[http_component]
fn handle_request(req: Request) -> anyhow::Result<impl IntoResponse> {
    let uri = req.uri().to_string();
    let method = req.method().clone();
    
    let path = uri.find("://")
        .map(|i| uri[i + 3..].find('/').map(|j| &uri[i + 3 + j..]).unwrap_or("/"))
        .unwrap_or(&uri);
    let path = path.split('?').next().unwrap_or(path);
    
    let conn = Connection::open_default()?;
    db::init(&conn)?;

    match (method, path) {
        (Method::Post, "/api/images") => upload_image(req, conn),
        (Method::Get, p) if p.starts_with("/api/image/") => {
            let id = p.trim_start_matches("/api/image/");
            get_image(id, conn)
        },
        (Method::Get, p) if p.starts_with("/api/meta/") => {
            let id = p.trim_start_matches("/api/meta/");
            get_image_metadata(id, conn)
        },
        (Method::Get, "/upload.html") => {
            if !auth::check_basic_auth(&req)? {
                return Ok(Response::builder()
                    .status(401)
                    .header("WWW-Authenticate", "Basic realm=\"Image Upload\"")
                    .body(Bytes::from("Unauthorized"))
                    .build());
            }
            serve_upload_page()
        },
        (Method::Get, "/api/next") => get_next_image(conn),
        _ => Ok(Response::builder()
            .status(404)
            .body(Bytes::from("Not Found"))
            .build()),
    }
}

fn serve_upload_page() -> anyhow::Result<Response> {
    Ok(Response::builder()
        .status(200)
        .header("content-type", "text/html")
        .body(Bytes::from(include_str!("../www/upload.html")))
        .build())
}

fn upload_image(req: Request, conn: Connection) -> anyhow::Result<Response> {
    let boundary = req.header("content-type")
        .and_then(|v| v.as_str())
        .and_then(|v| v.split("boundary=").nth(1))
        .ok_or_else(|| anyhow::anyhow!("Missing boundary"))?;

    let mut multipart = Multipart::with_body(std::io::Cursor::new(req.body()), boundary);
    
    let mut image_data = Vec::new();
    let mut mime_type = String::new();
    let mut name = String::new();
    let mut description = String::new();

    while let Some(mut field) = multipart.read_entry()? {
        match &*field.headers.name {
            "image" => {
                mime_type = field.headers.content_type.map(|m| m.to_string()).unwrap_or("application/octet-stream".to_string());
                field.data.read_to_end(&mut image_data)?;
            },
            "name" => {
                field.data.read_to_string(&mut name)?;
            },
            "description" => {
                field.data.read_to_string(&mut description)?;
            },
            _ => {}
        }
    }

    if image_data.is_empty() || name.is_empty() || description.is_empty() {
        return Ok(Response::builder()
            .status(400)
            .body(Bytes::from("Missing required fields"))
            .build());
    }
    
    let mut hasher = Sha256::new();
    hasher.update(&image_data);
    let hash = hex::encode(hasher.finalize());
    
    let id = Uuid::new_v4().to_string();
    
    db::insert_image(&conn, &id, &image_data, &mime_type, &hash, &name, &description)?;
    
    let response = UploadResponse { id, hash };
    let body = serde_json::to_string(&response)?;
    
    Ok(Response::builder()
        .status(201)
        .header("content-type", "application/json")
        .body(Bytes::from(body))
        .build())
}

fn get_image(id: &str, conn: Connection) -> anyhow::Result<Response> {
    if let Some((image, mime_type, hash)) = db::get_image_data(&conn, id)? {
        Ok(Response::builder()
            .status(200)
            .header("content-type", mime_type)
            .header("etag", hash)
            .body(Bytes::from(image))
            .build())
    } else {
        Ok(Response::builder()
            .status(404)
            .body(Bytes::from("Image not found"))
            .build())
    }
}

fn get_image_metadata(id: &str, conn: Connection) -> anyhow::Result<Response> {
    if let Some(metadata) = db::get_image_metadata(&conn, id)? {
        let body = serde_json::to_string(&metadata)?;
        Ok(Response::builder()
            .status(200)
            .header("content-type", "application/json")
            .body(Bytes::from(body))
            .build())
    } else {
        Ok(Response::builder()
            .status(404)
            .body(Bytes::from("Image not found"))
            .build())
    }
}

fn get_next_image(conn: Connection) -> anyhow::Result<Response> {
    if let Some(metadata) = db::get_random_image(&conn)? {
        let body = serde_json::to_string(&metadata)?;
        Ok(Response::builder()
            .status(200)
            .header("content-type", "application/json")
            .body(Bytes::from(body))
            .build())
    } else {
        Ok(Response::builder()
            .status(404)
            .body(Bytes::from("No images found"))
            .build())
    }
}
