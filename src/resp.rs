///! Semantic HTTP response helpers.
use bytes::Bytes;
use http::{HttpTryFrom, StatusCode};
use http_service::Body;
use tide::Response;

/// Set a json body and generate `Response`
pub fn json<S, T: serde::Serialize>(status: S, t: T) -> Response
where
    StatusCode: HttpTryFrom<S>,
{
    http::Response::builder()
        .status(status)
        .header("Content-Type", "text/html")
        .body(Body::from(serde_json::to_vec(&t).unwrap()))
        .unwrap()
}

/// Set a html body and generate `Response`
pub fn html<S, T: Into<Bytes> + Send>(status: S, t: T) -> Response
where
    StatusCode: HttpTryFrom<S>,
{
    http::Response::builder()
        .status(status)
        .header("Content-Type", "text/html")
        .body(Body::from(t))
        .unwrap()
}
