use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use bytes::Buf;

use funcky::{DropDir, Error as MgError, FunckManager};

use futures::StreamExt;

use snafu::{ResultExt, Snafu};

use tempfile::NamedTempFile;

use warp::{
    http::{header::HeaderName, HeaderValue, StatusCode},
    hyper::Body,
    reply,
};

use super::message::{ErrorMessage, Message};
use super::zip;

#[derive(Debug, Snafu)]
pub enum Error {
    FailedToReadBody {
        source: warp::Error,
    },

    FailedToWriteBody {
        source: io::Error,
    },

    FailedToDeleteSourceBundle {
        source: io::Error,
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

    InvalidHeader,
}

fn header_name(name: &str) -> Result<HeaderName, Error> {
    HeaderName::from_str(name).map_err(|_e| Error::InvalidHeader)
}

fn header_val(name: &str) -> Result<HeaderValue, Error> {
    HeaderValue::from_str(name).map_err(|_e| Error::InvalidHeader)
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
    fs::write(&dst_zip_path.path(), body.bytes()).context(FailedToWriteBody)?;

    // Extract zip file.
    let tgt_dir = DropDir::new(
        manager.cfg.tmp_dir.join(project_name),
        project_name.to_string_lossy().as_ref(),
    )
    .context(FailedToWriteBody)?;

    log::debug!(
        "unzip {} => {}",
        dst_zip_path.path().display(),
        tgt_dir.path().display()
    );

    zip::unzip(&dst_zip_path.path(), &tgt_dir.path()).unwrap();

    // Delete zip file.
    fs::remove_file(dst_zip_path).context(FailedToDeleteSourceBundle)?;

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
            Ok(http_resp)
        }
        Err(e) => Err(warp::reject::custom(e)),
    }
}
