use std::sync::Arc;

use warp::Filter;

use super::handlers;
use super::response::handle_error;
use crate::funcky::FunckManager;

const ADD_FUNCTION_ROUTE_PATH: &str = "_funck_add";
const STAT_ROUTE_PATH: &str = "_stat";

fn with_manager(
    manager: Arc<FunckManager>,
) -> impl Filter<Extract = (Arc<FunckManager>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || manager.clone())
}

pub fn all(
    manager: Arc<FunckManager>,
) -> impl Filter<Extract = impl ::warp::Reply, Error = warp::Rejection> + Clone {
    add_function(manager.clone())
        .or(call_arbitrary(manager.clone()))
        .or(stat(manager))
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
        .recover(handle_error)
}

fn call_arbitrary(
    manager: Arc<FunckManager>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post()
        .and(with_manager(manager))
        .and(warp::body::content_length_limit(1024))
        .and(warp::body::bytes())
        .and(warp::path::path("call"))
        .and(warp::path::tail())
        .and_then(handlers::call)
        .recover(handle_error)
}

fn stat(
    manager: Arc<FunckManager>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path(STAT_ROUTE_PATH))
        .and(with_manager(manager))
        .and_then(handlers::stat)
}
