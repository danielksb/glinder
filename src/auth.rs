use spin_sdk::http::Request;
use spin_sdk::variables;
use base64::prelude::*;
use anyhow::Result;

pub fn check_basic_auth(req: &Request) -> Result<bool> {
    let header = req.header("Authorization")
        .and_then(|h| h.as_str())
        .unwrap_or("");
    
    if !header.starts_with("Basic ") {
        return Ok(false);
    }
    
    let encoded = &header[6..];
    let decoded = BASE64_STANDARD.decode(encoded).unwrap_or_default();
    let credentials = String::from_utf8(decoded).unwrap_or_default();
    
    let parts: Vec<&str> = credentials.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Ok(false);
    }
    
    let username = variables::get("username")?;
    let password = variables::get("password")?;
    
    Ok(parts[0] == username && parts[1] == password)
}
