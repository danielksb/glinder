mod db;
mod models;
mod auth;

use spin_sdk::http::responses::not_found;
use spin_sdk::http::{Params, Request, Response, Router};
use spin_sdk::http_component;
use spin_sdk::sqlite::Connection;
use uuid::Uuid;
use sha2::{Sha256, Digest};
use bytes::Bytes;
use multipart::server::Multipart;
use std::io::Read;
use models::UploadResponse;

#[http_component]
fn handle_request(req: Request) -> Response {
    let mut router = Router::new();
    router.post("/api/images", upload_image);
    router.delete("/api/images", delete_image);
    router.get("/api/image/:id", get_image);
    router.get("/api/meta/:id", get_image_metadata);
    router.get("/upload.html", serve_upload_page);
    router.get("/api/next", get_next_image);
    router.any("/*", handle_not_found);
    router.handle(req)
}

fn handle_not_found(_: Request, _: Params) -> anyhow::Result<Response> {
    Ok(not_found())
}

fn serve_upload_page(req: Request, _: Params) -> anyhow::Result<Response> {
    if !auth::check_basic_auth(&req)? {
                return Ok(Response::builder()
                    .status(401)
                    .header("WWW-Authenticate", "Basic realm=\"Image Upload\"")
                    .body(Bytes::from("Unauthorized"))
                    .build());
            }
    Ok(Response::builder()
        .status(200)
        .header("content-type", "text/html")
        .body(Bytes::from(include_str!("../www/upload.html")))
        .build())
}

fn upload_image(req: Request, _: Params) -> anyhow::Result<Response> {
    let conn = Connection::open_default()?;
    db::init(&conn)?;

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

fn get_image(_: Request, params: Params) -> anyhow::Result<Response> {
    let conn = Connection::open_default()?;
    db::init(&conn)?;

    let id = match params.get("id") {
        Some(v) => v,
        None => return Ok(not_found()),
    };

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

fn delete_image(req: Request, params: Params) -> anyhow::Result<Response> {
    let conn = Connection::open_default()?;
    db::init(&conn)?;
    
    // Delete an image by id - requires Basic auth (same protection as upload UI)
    if !auth::check_basic_auth(&req)? {
        return Ok(Response::builder()
            .status(401)
            .header("WWW-Authenticate", "Basic realm=\"Image Upload\"")
            .body(Bytes::from("Unauthorized"))
            .build());
    }
    let id = match params.get("id") {
        Some(v) => v,
        None => return Ok(not_found()),
    };

    if db::delete_image(&conn, id)? {
        Ok(Response::builder()
            .status(204)
            .body(Bytes::from(""))
            .build())
    } else {
        Ok(Response::builder()
            .status(404)
            .body(Bytes::from("Image not found"))
            .build())
    }
}

fn get_image_metadata(_: Request, params: Params) -> anyhow::Result<Response> {
    let conn = Connection::open_default()?;
    db::init(&conn)?;
    
    let id = match params.get("id") {
        Some(v) => v,
        None => return Ok(not_found()),
    };

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

fn get_next_image(_: Request, _: Params) -> anyhow::Result<Response> {
    let conn = Connection::open_default()?;
    db::init(&conn)?;

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
