use std::fs;
use std::path::Path;
use std::sync::Arc;

use bytes::Buf;

use futures::StreamExt;

use tempfile::TempDir;

use warp::reply;

use super::message::Message;
use super::zip;
use crate::funcky::{Error as MgError, FunckManager};

impl warp::reject::Reject for MgError {}

async fn add_part(manager: Arc<FunckManager>, mut part: warp::multipart::Part) {
    let body = part.data().await.unwrap().unwrap(); // TODO: Handle
    let dir = TempDir::new_in(&manager.cfg.tmp_dir).unwrap(); // TODO: Handle.

    let fname = Path::new(part.filename().unwrap());
    let project_name = fname.file_stem().unwrap();

    // Save zip file.
    let dst_zip_path = dir.path().join("tmp.zip");
    log::debug!("writing source zip to {}", dst_zip_path.display());
    fs::write(&dst_zip_path, body.bytes()).unwrap(); // TODO: Handle.

    // Extract zip file.
    let tgt_dir = dir.path().join(project_name);
    log::debug!("unzip {} => {}", dst_zip_path.display(), tgt_dir.display());
    zip::unzip(&dst_zip_path, &tgt_dir).unwrap();

    // Add to manager.
    manager.add(tgt_dir).unwrap();
}

// TODO: Add content-type to indicate file extension (zip, tar.gz, tar.xz)
pub async fn add(
    manager: Arc<FunckManager>,
    mut form_data: warp::multipart::FormData,
) -> Result<impl warp::Reply, std::convert::Infallible> {
    log::info!("POST/add");

    while let Some(Ok(part)) = form_data.next().await {
        if part.name() == "src" {
            add_part(manager.clone(), part).await;
        }
    }
    Ok(reply::json(&Message::new("OK")))
}

pub async fn call(
    manager: Arc<FunckManager>,
    path: warp::path::Tail,
) -> Result<impl warp::Reply, warp::Rejection> {
    log::info!("POST{}", path.as_str());
    match manager.call(path.as_str()) {
        Ok(_) => Ok(reply::json(&Message::new("OK"))),
        Err(e) => Err(warp::reject::custom(e)),
    }
}
