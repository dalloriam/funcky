use std::fs;
use std::path::Path;
use std::sync::Arc;

use bytes::Buf;

use futures::StreamExt;

use snafu::{ResultExt, Snafu};

use tempfile::NamedTempFile;

use warp::{http::StatusCode, reply};

use super::message::{ErrorMessage, Message};
use super::zip;
use crate::funcky::{DropDir, Error as MgError, FunckManager};

impl warp::reject::Reject for MgError {}

#[derive(Debug, Snafu)]
pub enum Error {
    FailedToReadBody {
        source: warp::Error,
    },
    #[snafu(display("{}", source))]
    ManagerAddError {
        source: MgError,
    },
    MissingPartData,

    #[snafu(display("{}", source))]
    CallError {
        source: MgError,
    },
}

async fn add_part(
    manager: Arc<FunckManager>,
    mut part: warp::multipart::Part,
) -> Result<(), Error> {
    let body = part
        .data()
        .await
        .ok_or(Error::MissingPartData)?
        .context(FailedToReadBody)?;

    let fname = Path::new(part.filename().unwrap());
    let project_name = fname.file_stem().unwrap();

    // TODO: Use a tempdir or a tempdir-like abstraction for the compilation request so that it
    // gets dropped once compilation is done.

    // Save zip file.
    let dst_zip_path = NamedTempFile::new_in(&manager.cfg.tmp_dir).unwrap();
    log::debug!("writing source zip to {}", dst_zip_path.path().display());
    fs::write(&dst_zip_path.path(), body.bytes()).unwrap(); // TODO: Handle.

    // Extract zip file.
    let tgt_dir = DropDir::new(manager.cfg.tmp_dir.join(project_name)).unwrap(); // TODO: Handle
    log::debug!(
        "unzip {} => {}",
        dst_zip_path.path().display(),
        tgt_dir.path().display()
    );

    zip::unzip(&dst_zip_path.path(), &tgt_dir.path()).unwrap();

    // Delete zip file.
    fs::remove_file(dst_zip_path).unwrap(); // TODO: Handle.

    // Add to manager.
    manager.add(tgt_dir).context(ManagerAddError)
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

pub async fn call(
    manager: Arc<FunckManager>,
    path: warp::path::Tail,
) -> Result<impl warp::Reply, warp::Rejection> {
    log::info!("POST/{}", path.as_str());
    match manager.call(path.as_str()) {
        Ok(_) => Ok(reply::json(&Message::new("OK"))),
        Err(e) => Err(warp::reject::custom(e)),
    }
}
