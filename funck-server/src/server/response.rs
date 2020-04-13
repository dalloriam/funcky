use executor::{Error as FnError, LoaderError};

use serde::Serialize;

use warp::http::StatusCode;
use warp::reject::Rejection;

use super::message::ErrorMessage;

pub fn get_serializable(err: &FnError) -> impl Serialize {
    match err {
        FnError::CallError {
            source: LoaderError::UnknownFunction { .. },
        } => ErrorMessage::new(err),
        FnError::CallError {
            source: LoaderError::CallError { source, .. },
        } => ErrorMessage::new(source),
        _ => ErrorMessage::new(&String::from("Internal Server Error")),
    }
}

pub fn get_status_code(err: &FnError) -> StatusCode {
    match err {
        FnError::CallError {
            source: LoaderError::UnknownFunction { .. },
        } => StatusCode::NOT_FOUND,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn handle_error(rejection: Rejection) -> Result<impl warp::Reply, warp::Rejection> {
    if let Some(err) = rejection.find::<FnError>() {
        Ok(warp::reply::with_status(
            warp::reply::json(&get_serializable(err)),
            get_status_code(err),
        ))
    } else {
        Err(rejection)
    }
}
