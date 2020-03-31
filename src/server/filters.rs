use std::sync::Arc;

use warp::Filter;

use super::handlers;
use super::response::handle_error;
use crate::funcky::FunckManager;

const ADD_FUNCTION_ROUTE_PATH: &str = "_funck_add";

fn with_manager(
    manager: Arc<FunckManager>,
) -> impl Filter<Extract = (Arc<FunckManager>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || manager.clone())
}

pub fn all(
    manager: Arc<FunckManager>,
) -> impl Filter<Extract = impl ::warp::Reply, Error = warp::Rejection> + Clone {
    add_function(manager.clone()).or(call_arbitrary(manager.clone()))
}

fn add_function(
    manager: Arc<FunckManager>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post()
        .and(warp::path(ADD_FUNCTION_ROUTE_PATH))
        .and(with_manager(manager))
        .and(warp::body::content_length_limit(100 * 1024)) // 100kb payload limit.
        .and(warp::multipart::form())
        .and_then(handlers::add)
        .recover(|error: warp::Rejection| handle_error(error))
}

fn call_arbitrary(
    manager: Arc<FunckManager>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post()
        .and(with_manager(manager))
        .and(warp::body::content_length_limit(1 * 1024))
        .and(warp::path::path("call"))
        .and(warp::path::tail())
        .and_then(handlers::call)
        .recover(|error: warp::Rejection| handle_error(error))
}