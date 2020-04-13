use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::{Context, Result};

use bytes::Buf;

use executor::{DropDir, FunckManager};

use futures::StreamExt;

use tempfile::NamedTempFile;

use warp::{
    http::{header::HeaderName, HeaderValue, StatusCode},
    hyper::Body,
    reply, Reply,
};

use super::message::{ErrorMessage, Message};
use super::zip;

fn header_name(name: &str) -> Result<HeaderName> {
    HeaderName::from_str(name).context("Invalid header key")
}

fn header_val(name: &str) -> Result<HeaderValue> {
    HeaderValue::from_str(name).context("Invalid header value")
}

async fn add_part(manager: Arc<FunckManager>, mut part: warp::multipart::Part) -> Result<()> {
    let body = part.data().await.unwrap()?; // TODO: Check for missing part.
    let fname = Path::new(part.filename().unwrap());
    let project_name = fname.file_stem().unwrap();

    // TODO: Use a tempdir or a tempdir-like abstraction for the compilation request so that it
    // gets dropped once compilation is done.

    // Save zip file.
    let dst_zip_path = NamedTempFile::new_in(&manager.cfg.tmp_dir).unwrap();
    log::debug!("writing source zip to {}", dst_zip_path.path().display());
    fs::write(&dst_zip_path.path(), body.bytes())?;

    // Extract zip file.
    let tgt_dir = DropDir::new(
        manager.cfg.tmp_dir.join(project_name),
        project_name.to_string_lossy().as_ref(),
    )?;

    log::debug!(
        "unzip {} => {}",
        dst_zip_path.path().display(),
        tgt_dir.path().display()
    );

    zip::unzip(&dst_zip_path.path(), &tgt_dir.path())?;

    // Delete zip file.
    fs::remove_file(dst_zip_path).context("Failed to delete source bundle")?;

    // Add to manager.
    manager.add(tgt_dir)?;

    Ok(())
}

// TODO: Add content-type to indicate file extension (zip, tar.gz, tar.xz)
pub async fn add(
    manager: Arc<FunckManager>,
    mut form_data: warp::multipart::FormData,
) -> Result<impl warp::Reply, warp::Rejection> {
    log::info!("POST/add");

    while let Some(Ok(part)) = form_data.next().await {
        if part.name() == "src" {
            if let Err(e) = add_part(manager.clone(), part).await {
                return Ok(reply::with_status(
                    reply::json(&ErrorMessage::new(&e)),
                    StatusCode::INTERNAL_SERVER_ERROR,
                ));
            }
        }
    }

    Ok(reply::with_status(
        reply::json(&Message::new("OK")),
        StatusCode::OK,
    ))
}

pub async fn stat(manager: Arc<FunckManager>) -> Result<impl warp::Reply, warp::Rejection> {
    log::info!("GET/stat");

    let stats = manager.stat();
    Ok(reply::json(&stats))
}

pub async fn call(
    manager: Arc<FunckManager>,
    body: bytes::Bytes,
    path: warp::path::Tail,
) -> Result<impl warp::Reply, warp::Rejection> {
    log::info!("POST/{}", path.as_str());

    let body_vec = if body.is_empty() {
        Vec::new()
    } else {
        body.bytes().to_vec()
    };

    let req = funck::Request::new(body_vec, HashMap::new());

    let function_defined = manager.has(path.as_str());
    if function_defined.is_err() || !function_defined.unwrap() {
        // Return a 404.
        return Ok(reply::with_status(
            reply::json(&ErrorMessage::new(&String::from("Funcktion not found"))).into_response(),
            StatusCode::NOT_FOUND,
        ));
    }

    match manager.call(path.as_str(), req) {
        Ok(resp) => {
            let body = Body::from(Vec::from(resp.body()));
            let mut http_resp = reply::Response::new(body);
            for (k, v) in resp.metadata().iter().filter_map(|(a, b)| {
                if let Ok(name) = header_name(a) {
                    if let Ok(val) = header_val(b) {
                        return Some((name, val));
                    }
                }
                log::warn!("skipped invalid header: [{}={}]", a, b);
                None
            }) {
                http_resp.headers_mut().insert(k, v);
            }
            Ok(reply::with_status(http_resp, StatusCode::OK))
        }
        Err(e) => {
            // Funcktion execution failed, return a 500.
            return Ok(reply::with_status(
                reply::json(&ErrorMessage::new(&e.to_string())).into_response(),
                StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    }
}
