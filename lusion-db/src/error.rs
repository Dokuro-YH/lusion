//! Error and Result module.
use failure::Fail;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "diesel error: {}", _0)]
    DieselError(diesel::result::Error),

    #[fail(display = "pool error: {}", _0)]
    PoolError(diesel::r2d2::PoolError),
}

impl From<diesel::r2d2::PoolError> for Error {
    fn from(err: diesel::r2d2::PoolError) -> Self {
        Error::PoolError(err)
    }
}

impl From<diesel::result::Error> for Error {
    fn from(err: diesel::result::Error) -> Self {
        Error::DieselError(err)
    }
}
