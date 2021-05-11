//! Filters, essentially how [`warp`] implements routes and middlewares.

use std::{convert::TryFrom, sync::Arc};

use warp::Filter;

use crate::{
    http::handlers,
    logic::{AuthenticateRequest, EnrollRequest, Logic, Signer, Verifier},
};

/// Pass the [`Arc`] to the handler.
fn with_arc<T>(
    val: Arc<T>,
) -> impl Filter<Extract = (Arc<T>,), Error = std::convert::Infallible> + Clone
where
    Arc<T>: Send,
{
    warp::any().map(move || Arc::clone(&val))
}

/// Extract the JSON body from the request, rejecting the excessive inputs size.
fn json_body<T>() -> impl Filter<Extract = (T,), Error = warp::Rejection> + Clone
where
    T: Send + for<'de> serde::de::Deserialize<'de>,
{
    // When accepting a body, we want a JSON body
    // (and to reject huge payloads)...
    warp::body::content_length_limit(1024 * 16).and(warp::body::json::<T>())
}

/// The root mount point with all the routes.
pub fn root<S, PK>(
    logic: Arc<Logic<S, PK>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
where
    S: Signer + Send + 'static,
    PK: Send + for<'a> TryFrom<&'a str> + Verifier + AsRef<[u8]> + Into<String>,
{
    enroll(logic.clone()).or(authenticate(logic))
}

/// POST /enroll with JSON body.
fn enroll<S, PK>(
    logic: Arc<Logic<S, PK>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
where
    S: Signer + Send + 'static,
    PK: Send + for<'a> TryFrom<&'a str>,
{
    warp::path!("enroll")
        .and(warp::post())
        .and(with_arc(logic))
        .and(json_body::<EnrollRequest>())
        .and_then(handlers::enroll)
}

/// POST /authenticate with JSON body.
fn authenticate<S, PK>(
    logic: Arc<Logic<S, PK>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
where
    S: Signer + Send + 'static,
    PK: Send + for<'a> TryFrom<&'a str> + Verifier + AsRef<[u8]> + Into<String>,
{
    warp::path!("authenticate")
        .and(warp::post())
        .and(with_arc(logic))
        .and(json_body::<AuthenticateRequest>())
        .and_then(handlers::authenticate)
}
