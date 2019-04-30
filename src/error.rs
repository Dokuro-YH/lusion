//! Errors.
use std::fmt::{self, Display};

use failure::{Backtrace, Context, Fail};
use http::StatusCode;
use http_service::{Body, Response};
use juniper::{FieldError, IntoFieldError};
use tide::response::IntoResponse;

pub use failure::ResultExt;
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// A list specifying general categories of application error.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Fail)]
pub enum ErrorKind {
    #[fail(display = "Failed to execute graphql")]
    GraphqlError,

    #[fail(display = "Failed to get connection")]
    DbPoolError,

    #[fail(display = "Database access error")]
    DbError,

    #[fail(display = "Not found")]
    NotFound,
}

/// Genernal error type.
#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

impl Error {
    pub fn kind(&self) -> ErrorKind {
        *self.inner.get_context()
    }

    pub fn status(&self) -> StatusCode {
        use self::ErrorKind::*;
        match self.kind() {
            NotFound => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl Fail for Error {
    fn cause(&self) -> Option<&Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Error {
        Error { inner }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = self.status();
        http::Response::builder()
            .status(status)
            .header("Content-Type", "application/json")
            .body(Body::empty())
            .unwrap()
    }
}

impl IntoFieldError for Error {
    fn into_field_error(self) -> FieldError {
        FieldError::new("Custom error", graphql_value!({ "cause": "tttttt"}))
    }
}
