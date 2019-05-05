//! HTTP Response.
pub use http::status::StatusCode;
pub use http_service::{Body, Response};
pub use tide::response::IntoResponse;

use bytes::Bytes;
use futures::Stream;
use http::HttpTryFrom;

/// Set a empty body and generate `Response`
pub fn empty<S>(status: S) -> Response
where
    StatusCode: HttpTryFrom<S>,
{
    http::Response::builder()
        .status(status)
        .body(Body::empty())
        .unwrap()
}

/// Set a stream body and generate `Response`
pub fn stream<S, T>(status: S, stream: T) -> Response
where
    StatusCode: HttpTryFrom<S>,
    T: Stream<Item = Result<Bytes, std::io::Error>> + Send + 'static,
{
    http::Response::builder()
        .status(status)
        .body(Body::from_stream(stream))
        .unwrap()
}

/// Set a json body and generate `Response`
pub fn json<S, T: serde::Serialize>(status: S, t: T) -> Response
where
    StatusCode: HttpTryFrom<S>,
{
    http::Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;

    #[test]
    fn test_empty() {
        let resp = empty(http::StatusCode::OK);
        assert_eq!(resp.status(), http::StatusCode::OK);

        let body = resp.read_body();
        assert_eq!(body, "");
    }

    #[test]
    fn test_stream() {
        let resp = stream(http::StatusCode::OK, Body::empty());
        assert_eq!(resp.status(), http::StatusCode::OK);

        let body = resp.read_body();
        assert_eq!(body, "");
    }

    #[test]
    fn test_json() {
        let resp = json(http::StatusCode::OK, json!({ "message": "test" }));
        assert_eq!(resp.status(), http::StatusCode::OK);

        let content_type = resp.headers().get(http::header::CONTENT_TYPE);
        assert_matches!(content_type, Some(content_type) => {
            assert_eq!(
                content_type,
                http::header::HeaderValue::from_static("application/json")
            );
        });

        let body = resp.read_body();
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(
            json,
            json!({
                "message": "test"
            })
        );
    }

    #[test]
    fn test_html() {
        let resp = html(http::StatusCode::OK, "<h1>Hello World</h1>");
        assert_eq!(resp.status(), http::StatusCode::OK);

        let content_type = resp.headers().get(http::header::CONTENT_TYPE);
        assert_matches!(content_type, Some(content_type) => {
            assert_eq!(
                content_type,
                http::header::HeaderValue::from_static("text/html")
            );
        });

        let body = resp.read_body();
        assert_eq!(body, "<h1>Hello World</h1>");
    }
}
