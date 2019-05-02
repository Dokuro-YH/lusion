//! Errors.
use std::fmt::{self, Display};

use failure::{Backtrace, Context, Fail};
use http::StatusCode;
use http_service::{Body, Response};
use juniper::{FieldError, IntoFieldError};
use tide::response::IntoResponse;

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub fn user_error<S: Into<String>>(msg: S) -> Error {
    let kind = ErrorKind::UserError(msg.into());
    Error {
        inner: Context::new(kind),
    }
}

/// A list specifying general categories of application error.
#[derive(Debug, Clone, Eq, PartialEq, Fail)]
pub enum ErrorKind {
    #[fail(display = "Database access error")]
    DbError,

    #[fail(display = "{}", _0)]
    UserError(String),
}

/// Genernal error type.
#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

impl Error {
    pub fn kind(&self) -> ErrorKind {
        self.inner.get_context().clone()
    }

    pub fn status(&self) -> StatusCode {
        use self::ErrorKind::*;
        match self.kind() {
            UserError(_) => StatusCode::BAD_REQUEST,
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

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = self.status();
        let payload = json!({ "message": format!("{}", self.kind()) });

        http::Response::builder()
            .status(status)
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_vec(&payload).unwrap()))
            .unwrap()
    }
}

impl IntoFieldError for Error {
    fn into_field_error(self) -> FieldError {
        FieldError::new(format!("{}", self.kind()), juniper::Value::Null)
    }
}

pub trait ResultExt<T, E> {
    fn kind(self, kind: ErrorKind) -> Result<T, Error>;

    fn db_error(self) -> Result<T, Error>;

    fn user_error<S: Into<String>>(self, msg: S) -> Result<T, Error>;
}

impl<T, E> ResultExt<T, E> for Result<T, E>
where
    E: Fail,
{
    fn kind(self, kind: ErrorKind) -> Result<T, Error> {
        self.map_err(|err| Error {
            inner: err.context(kind),
        })
    }

    fn db_error(self) -> Result<T, Error> {
        self.kind(ErrorKind::DbError)
    }

    fn user_error<S: Into<String>>(self, msg: S) -> Result<T, Error> {
        self.kind(ErrorKind::UserError(msg.into()))
    }
}
